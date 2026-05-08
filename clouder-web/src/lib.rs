mod api;
mod auth;
mod dashboard;
mod session;

use anyhow::Result;
use axum::http::{HeaderName, HeaderValue};
use axum::{Router, routing::get, routing::post};
use axum_extra::extract::cookie::Key;
use clouder_core::database::dashboard_sessions::DashboardSession;
use hkdf::Hkdf;
use sha2::Sha256;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_http::set_header::SetResponseHeaderLayer;
use tracing::{info, warn};

pub use clouder_core::config::AppState;

// Sweep expired sessions periodically so the table never grows unbounded
// while still letting deletes batch — 15 minutes balances cost and freshness.
const SESSION_CLEANUP_INTERVAL_SECS: u64 = 15 * 60;
// Reclaim per-IP rate-limiter state once a minute so stale buckets don't pin
// memory; matches `tower-governor`'s recommended retain cadence.
const LIMITER_RETAIN_INTERVAL_SECS: u64 = 60;
// HKDF "info" string for cookie key derivation. Namespaced + versioned so a
// future format change can swap the label without invalidating prior secrets.
const COOKIE_KEY_HKDF_INFO: &[u8] = b"clouder-web cookie signing key v1";
// Default rate limits per remote IP: enough headroom for normal dashboard
// browsing while shedding scripted abuse before it reaches handlers.
const DEFAULT_RATE_PER_SEC: u64 = 2;
const DEFAULT_RATE_BURST: u32 = 30;
// Stricter limit on the DM-send endpoint: every request is an outbound
// Discord API call, so we keep the bucket small to avoid getting bot-banned.
const DM_RATE_PER_SEC: u64 = 1;
const DM_RATE_BURST: u32 = 5;

#[derive(Clone)]
pub struct WebState {
    pub app_state: AppState,
    pub cookie_key: Key,
}

impl axum::extract::FromRef<WebState> for AppState {
    fn from_ref(state: &WebState) -> AppState {
        state.app_state.clone()
    }
}

impl axum::extract::FromRef<WebState> for Key {
    fn from_ref(state: &WebState) -> Key {
        state.cookie_key.clone()
    }
}

pub async fn run(app_state: AppState) -> Result<()> {
    info!("starting API: {}/api", app_state.config.web.bind_addr,);

    let key = derive_cookie_key(&app_state.config.web.session_secret);
    let state = WebState {
        app_state: app_state.clone(),
        cookie_key: key,
    };

    let cleanup_db = Arc::clone(&app_state.db);
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(Duration::from_secs(SESSION_CLEANUP_INTERVAL_SECS));
        loop {
            interval.tick().await;
            match DashboardSession::delete_expired(&cleanup_db).await {
                Ok(n) if n > 0 => info!("expired {} dashboard session(s)", n),
                Ok(_) => {}
                Err(e) => warn!("session cleanup failed: {}", e),
            }
        }
    });

    let security_headers = ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static(
                "default-src 'self'; \
                 img-src 'self' https://cdn.discordapp.com data:; \
                 script-src 'self'; \
                 style-src 'self'; \
                 connect-src 'self'; \
                 frame-ancestors 'none'; \
                 base-uri 'self'; \
                 form-action 'self' https://discord.com",
            ),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("geolocation=(), camera=(), microphone=()"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ));

    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(DEFAULT_RATE_PER_SEC)
            .burst_size(DEFAULT_RATE_BURST)
            .finish()
            .expect("governor config valid"),
    );
    let governor_limiter = governor_conf.limiter().clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(LIMITER_RETAIN_INTERVAL_SECS));
        loop {
            interval.tick().await;
            governor_limiter.retain_recent();
        }
    });
    let rate_limit = GovernorLayer::new(governor_conf);

    let dm_governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(DM_RATE_PER_SEC)
            .burst_size(DM_RATE_BURST)
            .finish()
            .expect("dm governor config valid"),
    );
    let dm_limiter = dm_governor_conf.limiter().clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(LIMITER_RETAIN_INTERVAL_SECS));
        loop {
            interval.tick().await;
            dm_limiter.retain_recent();
        }
    });
    let dm_rate_limit = GovernorLayer::new(dm_governor_conf);

    let dm_route = Router::new()
        .route("/api/{user_id}", post(api::api_send_dm))
        .layer(dm_rate_limit)
        .with_state(state.clone());

    let app = Router::new()
        // pages
        .route("/", get(dashboard::index))
        .route("/login", get(dashboard::login_page))
        .route("/servers", get(dashboard::servers_page))
        .route("/dashboard/{guild_id}", get(dashboard::dashboard_redirect))
        .route(
            "/dashboard/{guild_id}/selfroles",
            get(dashboard::selfroles_page),
        )
        .route(
            "/dashboard/{guild_id}/welcome-goodbye",
            get(dashboard::welcome_goodbye_page),
        )
        .route("/dashboard/{guild_id}/about", get(dashboard::about_page))
        .route(
            "/dashboard/{guild_id}/mediaonly",
            get(dashboard::mediaonly_page),
        )
        .route("/dashboard/{guild_id}/uwufy", get(dashboard::uwufy_page))
        .route(
            "/dashboard/{guild_id}/reminders",
            get(dashboard::reminders_page),
        )
        .route("/profile", get(dashboard::profile_page))
        // auth
        .route("/auth/login", get(auth::login))
        .route("/auth/callback", get(auth::callback))
        .route("/auth/logout", get(auth::logout))
        // static assets
        .route("/static/style.css", get(static_css))
        .route("/static/app.js", get(static_js))
        // api
        .route("/api/guilds/refresh", post(api::api_guilds_refresh))
        .route("/api/guild/{guild_id}/channels", get(api::api_get_channels))
        .route("/api/guild/{guild_id}/roles", get(api::api_get_roles))
        .route(
            "/api/selfroles/{guild_id}",
            get(api::api_selfroles_list).post(api::api_selfroles_create),
        )
        .route(
            "/api/selfroles/{guild_id}/{config_id}",
            axum::routing::delete(api::api_selfroles_delete).put(api::api_selfroles_update),
        )
        .route(
            "/api/welcome-goodbye/{guild_id}/config",
            get(api::api_welcome_goodbye_get).post(api::api_welcome_goodbye_post),
        )
        .route(
            "/api/welcome-goodbye/{guild_id}/test/{message_type}",
            post(api::api_welcome_goodbye_test),
        )
        .route(
            "/api/mediaonly/{guild_id}",
            get(api::api_mediaonly_get).post(api::api_mediaonly_post),
        )
        .route(
            "/api/mediaonly/{guild_id}/{channel_id}",
            axum::routing::delete(api::api_mediaonly_delete).put(api::api_mediaonly_put),
        )
        .route("/api/guild/{guild_id}/about", get(api::api_about_get))
        .route(
            "/api/guild/{guild_id}/config",
            get(api::api_guild_config_get).post(api::api_guild_config_post),
        )
        .route(
            "/api/uwufy/{guild_id}",
            get(api::api_uwufy_get).delete(api::api_uwufy_disable_all),
        )
        .route(
            "/api/uwufy/{guild_id}/{user_id}",
            axum::routing::put(api::api_uwufy_toggle),
        )
        .route("/api/profile/regenerate-key", post(api::api_regenerate_key))
        .route(
            "/api/reminders/{guild_id}",
            get(api::api_reminders_get).post(api::api_reminders_post),
        )
        .route(
            "/api/reminders/{guild_id}/{config_id}/test",
            post(api::api_reminders_test),
        )
        // custom reminder endpoints
        .route(
            "/api/custom-reminders/{guild_id}",
            get(api::api_custom_reminders_list).post(api::api_custom_reminder_create),
        )
        .route(
            "/api/custom-reminders/{guild_id}/{reminder_id}",
            axum::routing::put(api::api_custom_reminder_update)
                .delete(api::api_custom_reminder_delete),
        )
        .route(
            "/api/custom-reminders/{guild_id}/{reminder_id}/test",
            post(api::api_custom_reminder_test),
        )
        // user-specific reminder endpoints
        .route(
            "/api/user/dm_reminders",
            get(api::api_user_dm_reminders_get).post(api::api_user_dm_reminders_post),
        )
        .route(
            "/api/user/subscriptions",
            get(api::api_user_subscriptions_get),
        )
        .route(
            "/api/user/subscribe/{config_id}",
            post(api::api_user_subscribe),
        )
        .route(
            "/api/user/unsubscribe/{config_id}",
            axum::routing::delete(api::api_user_unsubscribe),
        )
        .route(
            "/api/user/subscription/{id}",
            axum::routing::delete(api::api_user_subscription_delete),
        )
        .layer(rate_limit)
        .layer(security_headers)
        .with_state(state.clone())
        .merge(dm_route);

    let listener = tokio::net::TcpListener::bind(&state.app_state.config.web.bind_addr).await?;
    info!(
        "starting web dashboard: {}",
        &state.app_state.config.web.bind_addr,
    );
    info!("web base address: {}", &state.app_state.config.web.api_base);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}

/// Derives a 64-byte cookie signing key from the configured session secret
/// using HKDF-SHA256. The KDF turns short or low-entropy secrets into
/// uniformly distributed key material; a weak secret is still weak, but
/// truncation/repetition no longer leaks structure of the input.
fn derive_cookie_key(secret: &str) -> Key {
    let hk = Hkdf::<Sha256>::new(None, secret.as_bytes());
    let mut key_bytes = [0u8; 64];
    hk.expand(COOKIE_KEY_HKDF_INFO, &mut key_bytes)
        .expect("HKDF expand of 64 bytes is supported");
    Key::from(&key_bytes)
}

async fn static_css() -> impl axum::response::IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../static/style.css"),
    )
}

async fn static_js() -> impl axum::response::IntoResponse {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        include_str!("../static/app.js"),
    )
}
