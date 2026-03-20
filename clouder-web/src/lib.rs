mod api;
mod auth;
mod dashboard;
mod session;

use anyhow::Result;
use axum::{Router, routing::get, routing::post};
use axum_extra::extract::cookie::Key;
use tracing::info;

pub use clouder_core::config::AppState;

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

    let key = cookie_key_from_secret(&app_state.config.web.session_secret);
    let state = WebState {
        app_state,
        cookie_key: key,
    };

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
        .route(
            "/api/guilds/refresh",
            axum::routing::post(api::api_guilds_refresh),
        )
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
        .route("/api/{user_id}", post(api::api_send_dm))
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
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind(&state.app_state.config.web.bind_addr).await?;
    info!(
        "starting web dashboard: {}",
        &state.app_state.config.web.bind_addr,
    );
    info!("web base address: {}", &state.app_state.config.web.api_base);
    axum::serve(listener, app).await?;

    Ok(())
}

/// Derives a 64-byte `Key` from an arbitrary-length secret by repeating or truncating it.
fn cookie_key_from_secret(secret: &str) -> Key {
    let bytes = secret.as_bytes();
    let mut key_bytes = [0u8; 64];
    if bytes.len() >= 64 {
        key_bytes.copy_from_slice(&bytes[..64]);
    } else {
        for (i, b) in key_bytes.iter_mut().enumerate() {
            *b = bytes[i % bytes.len()];
        }
    }
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
