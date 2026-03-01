use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use clouder_core::config::AppState;
use serde_json::{Value, json};
use tracing::{error, info};

use crate::session::Auth;

pub async fn api_guilds_refresh(
    auth: Auth,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    match clouder_core::shared::refresh_guild_cache(&state, &auth.0.user_id, &auth.0.access_token)
        .await
    {
        Ok((guilds, updated)) => {
            let guild_list: Vec<Value> = guilds
                .iter()
                .map(|g| {
                    json!({
                        "id": g.id,
                        "name": g.name,
                        "icon": g.icon,
                    })
                })
                .collect();
            Ok(Json(json!({ "guilds": guild_list, "updated": updated })))
        }
        Err(e) => {
            error!("failed to refresh guild cache: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_get_channels(
    _auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::get_guild_channels(&state, guild_id_u64).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("failed to get channels: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_get_roles(
    _auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::get_guild_roles(&state, guild_id_u64).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("failed to get roles: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_selfroles_list(
    _auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::list_selfroles(&state, guild_id_u64).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("failed to list selfroles: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_selfroles_create(
    auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let user_id_u64: u64 = auth.0.user_id.parse().unwrap_or(0);
    match clouder_core::shared::create_selfrole(&state, guild_id_u64, user_id_u64, &payload).await {
        Ok(result) => {
            info!("selfrole created for guild {}", guild_id);
            Ok(Json(result))
        }
        Err(e) => {
            error!("failed to create selfrole: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_selfroles_update(
    auth: Auth,
    Path((guild_id, config_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let config_id_i64 = config_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let user_id_u64: u64 = auth.0.user_id.parse().unwrap_or(0);
    match clouder_core::shared::update_selfrole(
        &state,
        guild_id_u64,
        config_id_i64,
        user_id_u64,
        &payload,
    )
    .await
    {
        Ok(result) => {
            info!("selfrole {} updated for guild {}", config_id, guild_id);
            Ok(Json(result))
        }
        Err(e) => {
            error!("failed to update selfrole: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_selfroles_delete(
    _auth: Auth,
    Path((guild_id, config_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let config_id_i64 = config_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::delete_selfrole(&state, guild_id_u64, config_id_i64).await {
        Ok(result) => {
            info!("selfrole {} deleted for guild {}", config_id, guild_id);
            Ok(Json(result))
        }
        Err(e) => {
            error!("failed to delete selfrole: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_welcome_goodbye_get(
    _auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::get_welcome_goodbye_config(&state, guild_id_u64).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("failed to get welcome/goodbye config: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_welcome_goodbye_post(
    _auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::update_welcome_goodbye_config(&state, guild_id_u64, &payload).await
    {
        Ok(result) => {
            info!("welcome/goodbye config updated for guild {}", guild_id);
            Ok(Json(result))
        }
        Err(e) => {
            error!("failed to update welcome/goodbye config: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_welcome_goodbye_test(
    auth: Auth,
    Path((guild_id, message_type)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::send_test_welcome_message(
        &state,
        guild_id_u64,
        &message_type,
        &auth.0.user_id,
    )
    .await
    {
        Ok(_) => {
            info!("test {} message sent for guild {}", message_type, guild_id);
            Ok(Json(
                json!({ "success": true, "message": "test message sent" }),
            ))
        }
        Err(e) => {
            error!("failed to send test message: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_mediaonly_get(
    _auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::list_mediaonly_configs(&state, guild_id_u64).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("failed to get mediaonly configs: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_mediaonly_post(
    _auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let channel_id = payload
        .get("channel_id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::create_or_update_mediaonly_config(
        &state,
        guild_id_u64,
        channel_id,
        &payload,
    )
    .await
    {
        Ok(result) => {
            info!(
                "mediaonly config created for guild {} channel {}",
                guild_id, channel_id
            );
            Ok(Json(result))
        }
        Err(e) => {
            error!("failed to create mediaonly config: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_mediaonly_put(
    _auth: Auth,
    Path((guild_id, channel_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::create_or_update_mediaonly_config(
        &state,
        guild_id_u64,
        &channel_id,
        &payload,
    )
    .await
    {
        Ok(result) => {
            info!(
                "mediaonly config updated for guild {} channel {}",
                guild_id, channel_id
            );
            Ok(Json(result))
        }
        Err(e) => {
            error!("failed to update mediaonly config: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_mediaonly_delete(
    _auth: Auth,
    Path((guild_id, channel_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::delete_mediaonly_config(&state, guild_id_u64, &channel_id).await {
        Ok(result) => {
            info!(
                "mediaonly config deleted for guild {} channel {}",
                guild_id, channel_id
            );
            Ok(Json(result))
        }
        Err(e) => {
            error!("failed to delete mediaonly config: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_about_get(
    _auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::get_guild_about(&state, guild_id_u64).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("failed to get guild about: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_uwufy_get(
    _auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::list_uwufy_members(&state, guild_id_u64).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("failed to list uwufy members: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_uwufy_toggle(
    _auth: Auth,
    Path((guild_id, user_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let enabled = payload.get("enabled").and_then(|v| v.as_bool());
    match clouder_core::shared::toggle_uwufy_member(&state, guild_id_u64, &user_id, enabled).await {
        Ok(result) => {
            info!("uwufy toggled for user {} in guild {}", user_id, guild_id);
            Ok(Json(result))
        }
        Err(e) => {
            error!("failed to toggle uwufy: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_uwufy_disable_all(
    _auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::disable_all_uwufy(&state, guild_id_u64).await {
        Ok(result) => {
            info!("uwufy disabled for all in guild {}", guild_id);
            Ok(Json(result))
        }
        Err(e) => {
            error!("failed to disable all uwufy: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_send_dm(
    Path(user_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let key_str = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let record = clouder_core::DashboardUser::get_by_api_key(&state.db, key_str)
        .await
        .map_err(|e| {
            error!("failed to lookup api key: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if record.user_id != user_id {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let content = match payload.as_object() {
        Some(map) if map.len() == 1 => map.values().next().and_then(|v| v.as_str()),
        _ => payload.get("content").and_then(|v| v.as_str()),
    }
    .ok_or(StatusCode::BAD_REQUEST)?;

    let user_id_u64: u64 = user_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    clouder_core::shared::send_dm_to_user(&state.http, user_id_u64, content)
        .await
        .map_err(|e| {
            error!("failed to send dm to user {}: {}", user_id_u64, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(json!({ "success": true })))
}

pub async fn api_regenerate_key(
    auth: Auth,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let record = clouder_core::DashboardUser::regenerate_key(&state.db, &auth.0.user_id)
        .await
        .map_err(|e| {
            error!("failed to regenerate api key: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(json!({ "api_key": record.api_key })))
}
