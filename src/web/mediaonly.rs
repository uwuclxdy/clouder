use crate::{
    config::AppState,
    database::mediaonly::MediaOnlyConfig,
    logging::{error, info},
    utils::{get_bot_invite_url, get_guild_icon_url, get_guild_text_channels},
    web::session_extractor::extract_session_data,
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Redirect},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serenity::model::channel::ChannelType;

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaOnlyConfigRequest {
    pub channel_id: String,
    pub enabled: bool,
    pub allow_links: bool,
    pub allow_attachments: bool,
    pub allow_gifs: bool,
    pub allow_stickers: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaOnlyConfigUpdateRequest {
    pub allow_links: bool,
    pub allow_attachments: bool,
    pub allow_gifs: bool,
    pub allow_stickers: bool,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct MediaOnlyConfigDisplay {
    pub id: i64,
    pub channel_id: String,
    pub channel_name: String,
    pub enabled: bool,
    pub allow_links: bool,
    pub allow_attachments: bool,
    pub allow_gifs: bool,
    pub allow_stickers: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn get_mediaonly_page(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = extract_session_data(&headers)
        .await
        .map_err(|_| Redirect::temporary("/auth/login"))?;

    let user = session
        .1
        .ok_or_else(|| Redirect::temporary("/auth/login"))?;

    // Check if user has MANAGE_ROLES permission for this guild
    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild_info = user
        .guilds
        .iter()
        .find(|g| g.id == guild_id)
        .ok_or_else(|| Redirect::temporary("/"))?;

    // Get current configurations
    let configs = match MediaOnlyConfig::get_by_guild(&state.db, &guild_id).await {
        Ok(configs) => configs,
        Err(e) => {
            error!("get mediaonly configs: {}", e);
            return Err(Redirect::temporary("/"));
        }
    };

    // Get channels for the guild
    let channels = match get_guild_text_channels(&state.http, &guild_id).await {
        Ok(channels) => channels,
        Err(e) => {
            error!("get channels for {}: {}", guild_id, e);
            // Return empty channels list instead of redirecting, show friendly error in UI
            Vec::new()
        }
    };

    let guild_icon = get_guild_icon_url(&guild_info.id, guild_info.icon.as_ref());

    let invite_url = get_bot_invite_url(
        &state.config.web.oauth.client_id,
        Some(&state.config.web.oauth.redirect_uri),
    );

    // Build channels options HTML
    let mut channels_html = String::new();
    if channels.is_empty() {
        // Show helpful message when bot is not in guild
        channels_html.push_str(
            r#"<option value="" disabled>Bot not in server - please invite bot first</option>"#,
        );
    } else {
        for channel in &channels {
            channels_html.push_str(&format!(
                r#"<option value="{}"># {}</option>"#,
                channel.id, channel.name
            ));
        }
    }

    // Create display configs with channel names
    let mut display_configs = Vec::new();
    let guild_id_u64: u64 = guild_id.parse().unwrap_or(0);
    let discord_channels = state
        .http
        .get_channels(guild_id_u64.into())
        .await
        .unwrap_or_else(|_| Vec::new());

    for config in configs {
        let channel_name = discord_channels
            .iter()
            .find(|ch| ch.id.to_string() == config.channel_id)
            .map(|ch| ch.name.clone())
            .unwrap_or_else(|| "Unknown Channel".to_string());

        display_configs.push(MediaOnlyConfigDisplay {
            id: config.id,
            channel_id: config.channel_id,
            channel_name,
            enabled: config.enabled,
            allow_links: config.allow_links,
            allow_attachments: config.allow_attachments,
            allow_gifs: config.allow_gifs,
            allow_stickers: config.allow_stickers,
            created_at: config.created_at,
            updated_at: config.updated_at,
        });
    }

    // Build configuration JSON for JavaScript
    let configs_json = serde_json::to_string(&display_configs).unwrap_or_else(|_| "[]".to_string());

    let template = include_str!("templates/mediaonly_config.html")
        .replace("{{COMMON_CSS}}", include_str!("static/css/common.css"))
        .replace(
            "{{DASHBOARD_CSS}}",
            include_str!("static/css/dashboard.css"),
        )
        .replace("{{GUILD_NAME}}", &guild_info.name)
        .replace("{{GUILD_ICON}}", &guild_icon)
        .replace("{{INVITE_URL}}", &invite_url)
        .replace("{{GUILD_ID}}", &guild_id)
        .replace("{{CHANNELS_HTML}}", &channels_html)
        .replace(
            "{{BOT_MISSING}}",
            if channels.is_empty() { "true" } else { "false" },
        )
        .replace("{{CONFIGS_JSON}}", &configs_json);

    Ok(Html(template))
}

pub async fn list_configs(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let session = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = session.1.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let configs = match MediaOnlyConfig::get_by_guild(&state.db, &guild_id).await {
        Ok(configs) => configs,
        Err(e) => {
            error!("get mediaonly configs: {}", e);
            return Ok(Json(
                json!({"success": false, "error": "Failed to retrieve configurations"}),
            ));
        }
    };

    // Get channel names
    let guild_id_u64: u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let channels = match state.http.get_channels(guild_id_u64.into()).await {
        Ok(channels) => channels,
        Err(e) => {
            error!("get channels: {}", e);
            return Ok(Json(
                json!({"success": false, "error": "Failed to retrieve channels"}),
            ));
        }
    };

    let mut display_configs = Vec::new();
    for config in configs {
        let channel_name = channels
            .iter()
            .find(|ch| ch.id.to_string() == config.channel_id)
            .map(|ch| ch.name.clone())
            .unwrap_or_else(|| "Unknown Channel".to_string());

        display_configs.push(MediaOnlyConfigDisplay {
            id: config.id,
            channel_id: config.channel_id,
            channel_name,
            enabled: config.enabled,
            allow_links: config.allow_links,
            allow_attachments: config.allow_attachments,
            allow_gifs: config.allow_gifs,
            allow_stickers: config.allow_stickers,
            created_at: config.created_at,
            updated_at: config.updated_at,
        });
    }

    Ok(Json(json!({"success": true, "configs": display_configs})))
}

pub async fn create_or_update_config(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(request): Json<MediaOnlyConfigRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let session = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = session.1.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Validate channel exists and is a text channel
    let guild_id_u64: u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let channels = match state.http.get_channels(guild_id_u64.into()).await {
        Ok(channels) => channels,
        Err(e) => {
            error!("get channels: {}", e);
            return Ok(Json(
                json!({"success": false, "error": "Failed to validate channel"}),
            ));
        }
    };

    let channel_exists = channels
        .iter()
        .any(|ch| ch.id.to_string() == request.channel_id && matches!(ch.kind, ChannelType::Text));

    if !channel_exists {
        return Ok(Json(
            json!({"success": false, "error": "Invalid channel selected"}),
        ));
    }

    // Check if config already exists
    let existing_config =
        match MediaOnlyConfig::get_by_channel(&state.db, &guild_id, &request.channel_id).await {
            Ok(config) => config,
            Err(e) => {
                error!("check existing config: {}", e);
                return Ok(Json(
                    json!({"success": false, "error": "Failed to check existing configuration"}),
                ));
            }
        };

    if let Some(mut config) = existing_config {
        // Update existing config
        config.enabled = request.enabled;
        config.allow_links = request.allow_links;
        config.allow_attachments = request.allow_attachments;
        config.allow_gifs = request.allow_gifs;
        config.allow_stickers = request.allow_stickers;

        if let Err(e) = MediaOnlyConfig::update_permissions(
            &state.db,
            &guild_id,
            &request.channel_id,
            request.allow_links,
            request.allow_attachments,
            request.allow_gifs,
            request.allow_stickers,
        )
        .await
        {
            error!("update config: {}", e);
            return Ok(Json(
                json!({"success": false, "error": "Failed to update configuration"}),
            ));
        }
    } else {
        // Create new config
        if let Err(e) =
            MediaOnlyConfig::upsert(&state.db, &guild_id, &request.channel_id, request.enabled)
                .await
        {
            error!("create config: {}", e);
            return Ok(Json(
                json!({"success": false, "error": "Failed to create configuration"}),
            ));
        }

        // Update permissions for the new config
        if let Err(e) = MediaOnlyConfig::update_permissions(
            &state.db,
            &guild_id,
            &request.channel_id,
            request.allow_links,
            request.allow_attachments,
            request.allow_gifs,
            request.allow_stickers,
        )
        .await
        {
            error!("update permissions: {}", e);
            return Ok(Json(
                json!({"success": false, "error": "Failed to update permissions"}),
            ));
        }
    }

    info!(
        "mediaonly config updated: guild {} channel {}",
        guild_id, request.channel_id
    );
    Ok(Json(json!({"success": true})))
}

pub async fn update_permissions(
    Path((guild_id, channel_id)): Path<(String, String)>,
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(request): Json<MediaOnlyConfigUpdateRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let session = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = session.1.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check if config exists
    match MediaOnlyConfig::get_by_channel(&state.db, &guild_id, &channel_id).await {
        Ok(Some(mut config)) => {
            // Update enabled if provided
            if let Some(enabled) = request.enabled {
                config.enabled = enabled;
                if let Err(e) =
                    MediaOnlyConfig::upsert(&state.db, &guild_id, &channel_id, enabled).await
                {
                    error!("update enabled: {}", e);
                    return Ok(Json(
                        json!({"success": false, "error": "Failed to update enabled status"}),
                    ));
                }
            }
        }
        Ok(None) => {
            return Ok(Json(
                json!({"success": false, "error": "Configuration not found"}),
            ));
        }
        Err(e) => {
            error!("get config: {}", e);
            return Ok(Json(
                json!({"success": false, "error": "Failed to retrieve configuration"}),
            ));
        }
    };

    if let Err(e) = MediaOnlyConfig::update_permissions(
        &state.db,
        &guild_id,
        &channel_id,
        request.allow_links,
        request.allow_attachments,
        request.allow_gifs,
        request.allow_stickers,
    )
    .await
    {
        error!("update permissions: {}", e);
        return Ok(Json(
            json!({"success": false, "error": "Failed to update permissions"}),
        ));
    }

    info!(
        "mediaonly permissions updated: guild {} channel {}",
        guild_id, channel_id
    );
    Ok(Json(json!({"success": true})))
}

pub async fn delete_config(
    Path((guild_id, channel_id)): Path<(String, String)>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let session = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = session.1.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check if config exists
    match MediaOnlyConfig::get_by_channel(&state.db, &guild_id, &channel_id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return Ok(Json(
                json!({"success": false, "error": "Configuration not found"}),
            ));
        }
        Err(e) => {
            error!("get config: {}", e);
            return Ok(Json(
                json!({"success": false, "error": "Failed to retrieve configuration"}),
            ));
        }
    };

    if let Err(e) = MediaOnlyConfig::delete(&state.db, &guild_id, &channel_id).await {
        error!("delete config: {}", e);
        return Ok(Json(
            json!({"success": false, "error": "Failed to delete configuration"}),
        ));
    }

    info!(
        "mediaonly config deleted: guild {} channel {}",
        guild_id, channel_id
    );
    Ok(Json(json!({"success": true})))
}
