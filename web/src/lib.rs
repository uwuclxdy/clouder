mod auth;
mod config;
mod dashboard;
mod middleware;
mod models;

use anyhow::Result;
use axum::{routing::get, routing::post, Router};
use config::{AppState, Config};
use std::sync::Arc;
use tracing::info;

pub async fn run() -> Result<()> {
    info!("Starting clouder-web dashboard");

    let config = Arc::new(Config::from_env()?);
    info!("Configuration loaded");

    std::env::var("DISCORD_TOKEN").map_err(|_| anyhow::anyhow!("DISCORD_TOKEN not set"))?;

    let app_state = AppState { config };

    let app = Router::new()
        .route("/", get(auth::server_list))
        .route("/auth/login", get(auth::login))
        .route("/auth/callback", get(auth::callback))
        .route("/auth/logout", get(auth::logout))
        .route("/feature-request", get(dashboard::feature_request))
        .route("/user/settings", get(dashboard::user_settings))
        .route("/dashboard/{guild_id}", get(dashboard::guild_dashboard))
        .route(
            "/dashboard/{guild_id}/selfroles",
            get(dashboard::selfroles_list),
        )
        .route(
            "/dashboard/{guild_id}/selfroles/new",
            get(dashboard::selfroles_new),
        )
        .route(
            "/dashboard/{guild_id}/selfroles/{config_id}/edit",
            get(dashboard::selfroles_edit),
        )
        .route(
            "/dashboard/{guild_id}/welcome-goodbye",
            get(dashboard::welcome_goodbye_config),
        )
        .route(
            "/dashboard/{guild_id}/mediaonly",
            get(dashboard::mediaonly_config),
        )
        .route(
            "/api/guild/{guild_id}/channels",
            get(dashboard::api_get_channels),
        )
        .route("/api/guild/{guild_id}/roles", get(dashboard::api_get_roles))
        .route(
            "/api/selfroles/{guild_id}",
            get(dashboard::api_selfroles_list).post(dashboard::api_selfroles_create),
        )
        .route(
            "/api/selfroles/{guild_id}/{config_id}",
            axum::routing::delete(dashboard::api_selfroles_delete),
        )
        .route(
            "/api/welcome-goodbye/{guild_id}/config",
            get(dashboard::api_welcome_goodbye_get).post(dashboard::api_welcome_goodbye_post),
        )
        .route(
            "/api/welcome-goodbye/{guild_id}/test/{message_type}",
            post(dashboard::api_welcome_goodbye_test),
        )
        .route(
            "/api/mediaonly/{guild_id}",
            get(dashboard::api_mediaonly_get).post(dashboard::api_mediaonly_post),
        )
        .route(
            "/api/mediaonly/{guild_id}/{channel_id}",
            axum::routing::delete(dashboard::api_mediaonly_delete)
                .put(dashboard::api_mediaonly_put),
        )
        .layer(axum::middleware::from_fn(middleware::session_middleware))
        .with_state(app_state.clone());

    let addr = format!(
        "{}:{}",
        app_state.config.web.host, app_state.config.web.port
    );
    info!("Starting web server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
