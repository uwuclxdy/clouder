use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use clouder_core::config::AppState;
use serde_json::{Value, json};
use tracing::{error, info};

use crate::session::Auth;

async fn require_guild_access(
    state: &AppState,
    user_id: &str,
    guild_id: &str,
) -> Result<(), StatusCode> {
    use clouder_core::database::guild_cache::CachedGuild;
    if !CachedGuild::user_has_guild(&state.db, user_id, guild_id)
        .await
        .unwrap_or(false)
    {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

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
    auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
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
    auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
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
    auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
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
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
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
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
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
    auth: Auth,
    Path((guild_id, config_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
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
    auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
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
    auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
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
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
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
    auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
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
    auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
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
    auth: Auth,
    Path((guild_id, channel_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
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
    auth: Auth,
    Path((guild_id, channel_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
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
    auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::get_guild_about(&state, guild_id_u64).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("failed to get guild about: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_guild_config_get(
    auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::get_guild_config(&state, guild_id_u64).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("failed to get guild config: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_guild_config_post(
    auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    if let Some(tz) = payload.get("timezone").and_then(|v| v.as_str())
        && (tz.is_empty() || tz.len() > 64)
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    if let Some(prefix) = payload.get("command_prefix").and_then(|v| v.as_str())
        && (prefix.is_empty() || prefix.len() > 5)
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    if let Some(color) = payload.get("embed_color").and_then(|v| v.as_str())
        && !color.is_empty()
        && !color.starts_with('#')
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    match clouder_core::shared::update_guild_config(&state, guild_id_u64, &payload).await {
        Ok(result) => {
            info!("guild config updated for guild {}", guild_id);
            Ok(Json(result))
        }
        Err(e) => {
            error!("failed to update guild config: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_uwufy_get(
    auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
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
    auth: Auth,
    Path((guild_id, user_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
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
    auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
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

pub async fn api_reminders_get(
    auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::get_reminders_config(&state, guild_id_u64).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("failed to get reminders config: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_reminders_post(
    auth: Auth,
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
    let guild_id_u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::upsert_reminder_config(&state, guild_id_u64, &payload).await {
        Ok(result) => {
            info!("reminders config updated for guild {}", guild_id);
            Ok(Json(result))
        }
        Err(e) => {
            error!("failed to update reminders config: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_reminders_test(
    auth: Auth,
    Path((guild_id, config_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    require_guild_access(&state, &auth.0.user_id, &guild_id).await?;
    let config_id_i64: i64 = config_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_web_reminder_test(&state, config_id_i64).await {
        Ok(_) => {
            info!("reminder {} test fired for guild {}", config_id, guild_id);
            Ok(Json(json!({ "success": true, "message": "reminder sent" })))
        }
        Err(e) => {
            error!("failed to fire test reminder: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn clouder_web_reminder_test(state: &AppState, config_id: i64) -> Result<(), String> {
    use clouder_core::database::reminders::{
        ReminderConfig, ReminderLog, ReminderPingRole, ReminderType,
    };
    use serenity::all::{ChannelId, CreateEmbed, CreateEmbedFooter, CreateMessage};

    // fetch config row by id using helper
    let config: Option<ReminderConfig> = ReminderConfig::get_by_id(&state.db, config_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let config = config.ok_or("reminder config not found")?;
    let channel_id: u64 = config
        .channel_id
        .as_deref()
        .ok_or("no channel configured")?
        .parse()
        .map_err(|_| "invalid channel id")?;

    let ping_roles = ReminderPingRole::get_by_config(&state.db, config_id)
        .await
        .unwrap_or_default();
    let role_mentions: String = ping_roles
        .iter()
        .map(|r| format!("<@&{}>", r.role_id))
        .collect::<Vec<_>>()
        .join(" ");

    let mut msg = CreateMessage::new();
    if !role_mentions.is_empty() {
        msg = msg.content(&role_mentions);
    }

    let default_title = match config.reminder_type {
        ReminderType::Wysi => "7:27",
        ReminderType::Custom => "reminder",
    };
    let default_desc = match config.reminder_type {
        ReminderType::Wysi => "it's 7:27! when you see it :3 (test)".to_string(),
        ReminderType::Custom => "(test reminder)".to_string(),
    };

    if config.message_type == "embed" {
        let embed = CreateEmbed::new()
            .title(config.embed_title.as_deref().unwrap_or(default_title))
            .description(config.embed_description.as_deref().unwrap_or(&default_desc))
            .colour(config.embed_color.unwrap_or(0xFFFFFF) as u32)
            .footer(CreateEmbedFooter::new("clouder • test"));
        msg = msg.embed(embed);
    } else {
        let content = config.message_content.as_deref().unwrap_or(&default_desc);
        let full = if role_mentions.is_empty() {
            content.to_string()
        } else {
            format!("{} {}", role_mentions, content)
        };
        msg = CreateMessage::new().content(full);
    }

    state
        .http
        .send_message(ChannelId::new(channel_id), vec![], &msg)
        .await
        .map_err(|e| format!("failed to send: {}", e))?;

    let _ = ReminderLog::create(&state.db, config_id, "success", None, true, 0, 0).await;
    Ok(())
}

// user-specific reminder endpoints

pub async fn api_user_dm_reminders_get(
    auth: Auth,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = &auth.0.user_id;
    match clouder_core::shared::get_user_reminder_settings(&state, user_id).await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            error!("failed to get user settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_user_dm_reminders_post(
    auth: Auth,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = &auth.0.user_id;
    let timezone = payload
        .get("timezone")
        .and_then(|v| v.as_str())
        .unwrap_or("UTC");
    let dm_enabled = payload
        .get("dm_reminders_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    match clouder_core::shared::update_user_reminder_settings(&state, user_id, timezone, dm_enabled)
        .await
    {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            error!("failed to update user settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_user_subscriptions_get(
    auth: Auth,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = &auth.0.user_id;
    match clouder_core::shared::list_user_subscriptions(&state, user_id).await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            error!("failed to list subscriptions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_user_subscribe(
    auth: Auth,
    Path(config_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = &auth.0.user_id;
    let cid: i64 = config_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::add_user_subscription(&state, user_id, cid).await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            error!("failed to subscribe user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_user_unsubscribe(
    auth: Auth,
    Path(config_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = &auth.0.user_id;
    let cid: i64 = config_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    match clouder_core::shared::remove_user_subscription(&state, user_id, cid).await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            error!("failed to unsubscribe user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_user_subscription_delete(
    auth: Auth,
    Path(sub_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = &auth.0.user_id;
    let sid: i64 = sub_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    // ensure the subscription belongs to this user
    let subs =
        clouder_core::database::reminders::ReminderSubscription::get_by_user(&state.db, user_id)
            .await
            .map_err(|e| {
                error!("db error looking up subscriptions: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    if !subs.iter().any(|s| s.id == sid) {
        return Err(StatusCode::FORBIDDEN);
    }

    match clouder_core::shared::remove_subscription_by_id(&state, sid).await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            error!("failed to delete subscription: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
