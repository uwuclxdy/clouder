use crate::{
    config::AppState,
    database::welcome_goodbye::WelcomeGoodbyeConfig,
    logging::{error, info},
    utils::{
        get_bot_invite_url, get_default_embed_color, get_guild_icon_url, get_guild_text_channels,
        welcome_goodbye::{
            build_embed, replace_placeholders, validate_message_config, validate_url, EmbedConfig,
        },
    },
    web::{models::SessionUser, session_extractor::extract_session_data},
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Redirect},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serenity::builder::CreateMessage;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct WelcomeGoodbyeConfigRequest {
    pub welcome_enabled: bool,
    pub goodbye_enabled: bool,
    pub welcome_channel_id: Option<String>,
    pub goodbye_channel_id: Option<String>,
    pub welcome_message_type: String,
    pub goodbye_message_type: String,
    pub welcome_message_content: Option<String>,
    pub goodbye_message_content: Option<String>,
    // Welcome embed fields
    pub welcome_embed_title: Option<String>,
    pub welcome_embed_description: Option<String>,
    pub welcome_embed_color: Option<String>,
    pub welcome_embed_footer: Option<String>,
    pub welcome_embed_thumbnail: Option<String>,
    pub welcome_embed_image: Option<String>,
    pub welcome_embed_timestamp: bool,
    // Goodbye embed fields
    pub goodbye_embed_title: Option<String>,
    pub goodbye_embed_description: Option<String>,
    pub goodbye_embed_color: Option<String>,
    pub goodbye_embed_footer: Option<String>,
    pub goodbye_embed_thumbnail: Option<String>,
    pub goodbye_embed_image: Option<String>,
    pub goodbye_embed_timestamp: bool,
}

#[derive(Debug, Serialize)]
pub struct WelcomeGoodbyeConfigDisplay {
    pub guild_id: String,
    pub welcome_enabled: bool,
    pub goodbye_enabled: bool,
    pub welcome_channel_id: Option<String>,
    pub goodbye_channel_id: Option<String>,
    pub welcome_message_type: String,
    pub goodbye_message_type: String,
    pub welcome_message_content: Option<String>,
    pub goodbye_message_content: Option<String>,
    // Welcome embed fields
    pub welcome_embed_title: Option<String>,
    pub welcome_embed_description: Option<String>,
    pub welcome_embed_color: Option<i32>,
    pub welcome_embed_color_hex: String,
    pub welcome_embed_footer: Option<String>,
    pub welcome_embed_thumbnail: Option<String>,
    pub welcome_embed_image: Option<String>,
    pub welcome_embed_timestamp: bool,
    // Goodbye embed fields
    pub goodbye_embed_title: Option<String>,
    pub goodbye_embed_description: Option<String>,
    pub goodbye_embed_color: Option<i32>,
    pub goodbye_embed_color_hex: String,
    pub goodbye_embed_footer: Option<String>,
    pub goodbye_embed_thumbnail: Option<String>,
    pub goodbye_embed_image: Option<String>,
    pub goodbye_embed_timestamp: bool,
}

impl From<WelcomeGoodbyeConfig> for WelcomeGoodbyeConfigDisplay {
    fn from(config: WelcomeGoodbyeConfig) -> Self {
        let welcome_color_hex = config
            .welcome_embed_color
            .map(|c| format!("#{:06x}", c as u32))
            .unwrap_or_else(|| "#5865F2".to_string());

        let goodbye_color_hex = config
            .goodbye_embed_color
            .map(|c| format!("#{:06x}", c as u32))
            .unwrap_or_else(|| "#5865F2".to_string());

        Self {
            guild_id: config.guild_id,
            welcome_enabled: config.welcome_enabled,
            goodbye_enabled: config.goodbye_enabled,
            welcome_channel_id: config.welcome_channel_id,
            goodbye_channel_id: config.goodbye_channel_id,
            welcome_message_type: config.welcome_message_type,
            goodbye_message_type: config.goodbye_message_type,
            welcome_message_content: config.welcome_message_content,
            goodbye_message_content: config.goodbye_message_content,
            welcome_embed_title: config.welcome_embed_title,
            welcome_embed_description: config.welcome_embed_description,
            welcome_embed_color: config.welcome_embed_color,
            welcome_embed_color_hex: welcome_color_hex,
            welcome_embed_footer: config.welcome_embed_footer,
            welcome_embed_thumbnail: config.welcome_embed_thumbnail,
            welcome_embed_image: config.welcome_embed_image,
            welcome_embed_timestamp: config.welcome_embed_timestamp,
            goodbye_embed_title: config.goodbye_embed_title,
            goodbye_embed_description: config.goodbye_embed_description,
            goodbye_embed_color: config.goodbye_embed_color,
            goodbye_embed_color_hex: goodbye_color_hex,
            goodbye_embed_footer: config.goodbye_embed_footer,
            goodbye_embed_thumbnail: config.goodbye_embed_thumbnail,
            goodbye_embed_image: config.goodbye_embed_image,
            goodbye_embed_timestamp: config.goodbye_embed_timestamp,
        }
    }
}

pub async fn show_welcome_goodbye_config(
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

    // Get current configuration
    let config = match WelcomeGoodbyeConfig::get_config(&state.db, &guild_id).await {
        Ok(Some(config)) => config,
        Ok(None) => WelcomeGoodbyeConfig {
            guild_id: guild_id.clone(),
            ..Default::default()
        },
        Err(e) => {
            error!("get welcome/goodbye config: {}", e);
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

    let config_display = WelcomeGoodbyeConfigDisplay::from(config);

    // Build channels options HTML with selected values
    let mut channels_html = String::new();
    if channels.is_empty() {
        // Show helpful message when bot is not in guild
        channels_html.push_str(
            r#"<option value="" disabled>Bot not in server - please invite bot first</option>"#,
        );
    } else {
        for channel in &channels {
            let welcome_selected = config_display.welcome_channel_id.as_ref() == Some(&channel.id);
            let goodbye_selected = config_display.goodbye_channel_id.as_ref() == Some(&channel.id);

            channels_html.push_str(&format!(
                r#"<option value="{}" data-welcome-selected="{}" data-goodbye-selected="{}"># {}</option>"#,
                channel.id, welcome_selected, goodbye_selected, channel.name
            ));
        }
    }

    let template = include_str!("templates/welcome_goodbye_config.html")
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
        .replace(
            "{{WELCOME_ENABLED}}",
            if config_display.welcome_enabled {
                "checked"
            } else {
                ""
            },
        )
        .replace(
            "{{GOODBYE_ENABLED}}",
            if config_display.goodbye_enabled {
                "checked"
            } else {
                ""
            },
        )
        .replace(
            "{{WELCOME_MESSAGE_CONTENT}}",
            &config_display.welcome_message_content.unwrap_or_default(),
        )
        .replace(
            "{{GOODBYE_MESSAGE_CONTENT}}",
            &config_display.goodbye_message_content.unwrap_or_default(),
        )
        .replace(
            "{{WELCOME_EMBED_TITLE}}",
            &config_display.welcome_embed_title.unwrap_or_default(),
        )
        .replace(
            "{{WELCOME_EMBED_DESCRIPTION}}",
            &config_display.welcome_embed_description.unwrap_or_default(),
        )
        .replace(
            "{{WELCOME_EMBED_COLOR}}",
            &config_display.welcome_embed_color_hex,
        )
        .replace(
            "{{WELCOME_EMBED_FOOTER}}",
            &config_display.welcome_embed_footer.unwrap_or_default(),
        )
        .replace(
            "{{WELCOME_EMBED_THUMBNAIL}}",
            &config_display.welcome_embed_thumbnail.unwrap_or_default(),
        )
        .replace(
            "{{WELCOME_EMBED_IMAGE}}",
            &config_display.welcome_embed_image.unwrap_or_default(),
        )
        .replace(
            "{{WELCOME_EMBED_TIMESTAMP}}",
            if config_display.welcome_embed_timestamp {
                "checked"
            } else {
                ""
            },
        )
        .replace(
            "{{GOODBYE_EMBED_TITLE}}",
            &config_display.goodbye_embed_title.unwrap_or_default(),
        )
        .replace(
            "{{GOODBYE_EMBED_DESCRIPTION}}",
            &config_display.goodbye_embed_description.unwrap_or_default(),
        )
        .replace(
            "{{GOODBYE_EMBED_COLOR}}",
            &config_display.goodbye_embed_color_hex,
        )
        .replace(
            "{{GOODBYE_EMBED_FOOTER}}",
            &config_display.goodbye_embed_footer.unwrap_or_default(),
        )
        .replace(
            "{{GOODBYE_EMBED_THUMBNAIL}}",
            &config_display.goodbye_embed_thumbnail.unwrap_or_default(),
        )
        .replace(
            "{{GOODBYE_EMBED_IMAGE}}",
            &config_display.goodbye_embed_image.unwrap_or_default(),
        )
        .replace(
            "{{GOODBYE_EMBED_TIMESTAMP}}",
            if config_display.goodbye_embed_timestamp {
                "checked"
            } else {
                ""
            },
        )
        .replace(
            "{{WELCOME_MESSAGE_TYPE_EMBED_CHECKED}}",
            if config_display.welcome_message_type == "embed" {
                "checked"
            } else {
                ""
            },
        )
        .replace(
            "{{WELCOME_MESSAGE_TYPE_TEXT_CHECKED}}",
            if config_display.welcome_message_type == "text" {
                "checked"
            } else {
                ""
            },
        )
        .replace(
            "{{GOODBYE_MESSAGE_TYPE_EMBED_CHECKED}}",
            if config_display.goodbye_message_type == "embed" {
                "checked"
            } else {
                ""
            },
        )
        .replace(
            "{{GOODBYE_MESSAGE_TYPE_TEXT_CHECKED}}",
            if config_display.goodbye_message_type == "text" {
                "checked"
            } else {
                ""
            },
        )
        .replace(
            "{{WELCOME_MESSAGE_TYPE_EMBED_CLASS}}",
            if config_display.welcome_message_type == "embed" {
                "selected"
            } else {
                ""
            },
        )
        .replace(
            "{{WELCOME_MESSAGE_TYPE_TEXT_CLASS}}",
            if config_display.welcome_message_type == "text" {
                "selected"
            } else {
                ""
            },
        )
        .replace(
            "{{GOODBYE_MESSAGE_TYPE_EMBED_CLASS}}",
            if config_display.goodbye_message_type == "embed" {
                "selected"
            } else {
                ""
            },
        )
        .replace(
            "{{GOODBYE_MESSAGE_TYPE_TEXT_CLASS}}",
            if config_display.goodbye_message_type == "text" {
                "selected"
            } else {
                ""
            },
        )
        .replace(
            "{{WELCOME_TEXT_HIDE}}",
            if config_display.welcome_message_type == "text" {
                "show"
            } else {
                ""
            },
        )
        .replace(
            "{{WELCOME_EMBED_SHOW}}",
            if config_display.welcome_message_type == "embed" {
                "show"
            } else {
                ""
            },
        )
        .replace(
            "{{GOODBYE_TEXT_HIDE}}",
            if config_display.goodbye_message_type == "text" {
                "show"
            } else {
                ""
            },
        )
        .replace(
            "{{GOODBYE_EMBED_SHOW}}",
            if config_display.goodbye_message_type == "embed" {
                "show"
            } else {
                ""
            },
        )
        .replace(
            "{{WELCOME_CONFIG_DISPLAY}}",
            if config_display.welcome_enabled {
                "block"
            } else {
                "none"
            },
        )
        .replace(
            "{{GOODBYE_CONFIG_DISPLAY}}",
            if config_display.goodbye_enabled {
                "block"
            } else {
                "none"
            },
        );

    Ok(Html(template))
}

pub async fn save_welcome_goodbye_config(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(request): Json<WelcomeGoodbyeConfigRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let session = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = session.1.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Validate the configuration
    if request.welcome_enabled {
        if request.welcome_channel_id.is_none() {
            return Ok(Json(
                json!({"success": false, "error": "Welcome channel is required when welcome messages are enabled"}),
            ));
        }

        if let Err(e) = validate_message_config(
            &request.welcome_message_type,
            &request.welcome_message_content,
            &request.welcome_embed_title,
            &request.welcome_embed_description,
        ) {
            return Ok(Json(
                json!({"success": false, "error": format!("Welcome message validation error: {}", e)}),
            ));
        }

        // Validate URLs if provided
        if let Some(ref url) = request.welcome_embed_thumbnail
            && !url.is_empty()
            && !validate_url(url)
        {
            return Ok(Json(
                json!({"success": false, "error": "Invalid welcome thumbnail URL"}),
            ));
        }
        if let Some(ref url) = request.welcome_embed_image
            && !url.is_empty()
            && !validate_url(url)
        {
            return Ok(Json(
                json!({"success": false, "error": "Invalid welcome image URL"}),
            ));
        }
    }

    if request.goodbye_enabled {
        if request.goodbye_channel_id.is_none() {
            return Ok(Json(
                json!({"success": false, "error": "Goodbye channel is required when goodbye messages are enabled"}),
            ));
        }

        if let Err(e) = validate_message_config(
            &request.goodbye_message_type,
            &request.goodbye_message_content,
            &request.goodbye_embed_title,
            &request.goodbye_embed_description,
        ) {
            return Ok(Json(
                json!({"success": false, "error": format!("Goodbye message validation error: {}", e)}),
            ));
        }

        // Validate URLs if provided
        if let Some(ref url) = request.goodbye_embed_thumbnail
            && !url.is_empty()
            && !validate_url(url)
        {
            return Ok(Json(
                json!({"success": false, "error": "Invalid goodbye thumbnail URL"}),
            ));
        }
        if let Some(ref url) = request.goodbye_embed_image
            && !url.is_empty()
            && !validate_url(url)
        {
            return Ok(Json(
                json!({"success": false, "error": "Invalid goodbye image URL"}),
            ));
        }
    }

    // Convert hex colors to integers
    let welcome_embed_color = request
        .welcome_embed_color
        .as_ref()
        .and_then(|hex| hex.strip_prefix('#'))
        .and_then(|hex| i32::from_str_radix(hex, 16).ok());

    let goodbye_embed_color = request
        .goodbye_embed_color
        .as_ref()
        .and_then(|hex| hex.strip_prefix('#'))
        .and_then(|hex| i32::from_str_radix(hex, 16).ok());

    let config = WelcomeGoodbyeConfig {
        guild_id: guild_id.clone(),
        welcome_enabled: request.welcome_enabled,
        goodbye_enabled: request.goodbye_enabled,
        welcome_channel_id: request.welcome_channel_id,
        goodbye_channel_id: request.goodbye_channel_id,
        welcome_message_type: request.welcome_message_type,
        goodbye_message_type: request.goodbye_message_type,
        welcome_message_content: request.welcome_message_content,
        goodbye_message_content: request.goodbye_message_content,
        welcome_embed_title: request.welcome_embed_title,
        welcome_embed_description: request.welcome_embed_description,
        welcome_embed_color,
        welcome_embed_footer: request.welcome_embed_footer,
        welcome_embed_thumbnail: request.welcome_embed_thumbnail,
        welcome_embed_image: request.welcome_embed_image,
        welcome_embed_timestamp: request.welcome_embed_timestamp,
        goodbye_embed_title: request.goodbye_embed_title,
        goodbye_embed_description: request.goodbye_embed_description,
        goodbye_embed_color,
        goodbye_embed_footer: request.goodbye_embed_footer,
        goodbye_embed_thumbnail: request.goodbye_embed_thumbnail,
        goodbye_embed_image: request.goodbye_embed_image,
        goodbye_embed_timestamp: request.goodbye_embed_timestamp,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    match WelcomeGoodbyeConfig::upsert_config(&state.db, &config).await {
        Ok(_) => {
            info!("welcome/goodbye config updated: guild {}", guild_id);
            Ok(Json(json!({"success": true})))
        }
        Err(e) => {
            error!("save welcome/goodbye config: {}", e);
            Ok(Json(
                json!({"success": false, "error": "Failed to save configuration"}),
            ))
        }
    }
}

pub async fn send_test_welcome(
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

    // Get the current configuration
    let config = match WelcomeGoodbyeConfig::get_config(&state.db, &guild_id).await {
        Ok(Some(config)) => config,
        Ok(None) => {
            return Ok(Json(json!({
                "success": false,
                "error": "Welcome messages are not configured for this server"
            })));
        }
        Err(e) => {
            error!("get welcome/goodbye config: {}", e);
            return Ok(Json(json!({
                "success": false,
                "error": "Failed to retrieve configuration"
            })));
        }
    };

    if !config.welcome_enabled {
        return Ok(Json(json!({
            "success": false,
            "error": "Welcome messages are not enabled"
        })));
    }

    if config.welcome_channel_id.is_none() {
        return Ok(Json(json!({
            "success": false,
            "error": "Welcome channel is not configured"
        })));
    }

    // Send test welcome message using the logged-in user as test subject
    match send_test_message_to_channel(&state, &config, &user, &guild_id, "welcome").await {
        Ok(_) => {
            info!("test welcome: guild {} user {}", guild_id, user.user.id);
            Ok(Json(json!({"success": true})))
        }
        Err(e) => {
            error!("send test welcome: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": format!("Failed to send test message: {}", e)
            })))
        }
    }
}

pub async fn send_test_goodbye(
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

    // Get the current configuration
    let config = match WelcomeGoodbyeConfig::get_config(&state.db, &guild_id).await {
        Ok(Some(config)) => config,
        Ok(None) => {
            return Ok(Json(json!({
                "success": false,
                "error": "Goodbye messages are not configured for this server"
            })));
        }
        Err(e) => {
            error!("get welcome/goodbye config: {}", e);
            return Ok(Json(json!({
                "success": false,
                "error": "Failed to retrieve configuration"
            })));
        }
    };

    if !config.goodbye_enabled {
        return Ok(Json(json!({
            "success": false,
            "error": "Goodbye messages are not enabled"
        })));
    }

    if config.goodbye_channel_id.is_none() {
        return Ok(Json(json!({
            "success": false,
            "error": "Goodbye channel is not configured"
        })));
    }

    // Send test goodbye message using the logged-in user as test subject
    match send_test_message_to_channel(&state, &config, &user, &guild_id, "goodbye").await {
        Ok(_) => {
            info!("test goodbye: guild {} user {}", guild_id, user.user.id);
            Ok(Json(json!({"success": true})))
        }
        Err(e) => {
            error!("send test goodbye: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": format!("Failed to send test message: {}", e)
            })))
        }
    }
}

pub async fn get_live_preview(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
    Json(_request): Json<HashMap<String, serde_json::Value>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let session = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = session.1.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    // For now, return a simple preview placeholder
    Ok(Json(json!({
        "success": true,
        "preview": "Live preview functionality coming soon"
    })))
}

async fn send_test_message_to_channel(
    state: &AppState,
    config: &WelcomeGoodbyeConfig,
    user: &SessionUser,
    guild_id: &str,
    message_type: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let guild_id_u64: u64 = guild_id.parse()?;

    // Get guild information for member count
    let guild = state.http.get_guild(guild_id_u64.into()).await?;

    // Create placeholders using the logged-in user
    let mut placeholders = HashMap::new();
    placeholders.insert("user".to_string(), format!("<@{}>", user.user.id));
    placeholders.insert("username".to_string(), user.user.username.clone());
    placeholders.insert("server".to_string(), guild.name.clone());
    placeholders.insert(
        "member_count".to_string(),
        guild.approximate_member_count.unwrap_or(0).to_string(),
    );
    placeholders.insert("user_id".to_string(), user.user.id.clone());
    placeholders.insert("join_date".to_string(), "2024-01-01".to_string()); // Placeholder date

    let (
        channel_id,
        msg_type,
        content,
        embed_title,
        embed_desc,
        embed_color,
        embed_footer,
        embed_thumb,
        embed_image,
        embed_timestamp,
    ) = match message_type {
        "welcome" => (
            config.welcome_channel_id.as_ref().unwrap(),
            &config.welcome_message_type,
            &config.welcome_message_content,
            &config.welcome_embed_title,
            &config.welcome_embed_description,
            config.welcome_embed_color,
            &config.welcome_embed_footer,
            &config.welcome_embed_thumbnail,
            &config.welcome_embed_image,
            config.welcome_embed_timestamp,
        ),
        "goodbye" => (
            config.goodbye_channel_id.as_ref().unwrap(),
            &config.goodbye_message_type,
            &config.goodbye_message_content,
            &config.goodbye_embed_title,
            &config.goodbye_embed_description,
            config.goodbye_embed_color,
            &config.goodbye_embed_footer,
            &config.goodbye_embed_thumbnail,
            &config.goodbye_embed_image,
            config.goodbye_embed_timestamp,
        ),
        _ => return Err("Invalid message type".into()),
    };

    let channel_id_u64: u64 = channel_id.parse()?;

    match msg_type.as_str() {
        "embed" => {
            let default_color = get_default_embed_color(state).0 as u64;

            let embed_config = EmbedConfig {
                title: embed_title,
                description: embed_desc,
                color: embed_color,
                footer: embed_footer,
                thumbnail: embed_thumb,
                image: embed_image,
                timestamp: embed_timestamp,
                default_color,
            };

            let embed = build_embed(&embed_config, &placeholders);
            let message = CreateMessage::new().embed(embed);
            state
                .http
                .send_message(channel_id_u64.into(), Vec::new(), &message)
                .await?;
        }
        "text" => {
            if let Some(content) = content {
                let processed_content = replace_placeholders(content, &placeholders);
                if !processed_content.trim().is_empty() {
                    let message = CreateMessage::new().content(processed_content);
                    state
                        .http
                        .send_message(channel_id_u64.into(), Vec::new(), &message)
                        .await?;
                }
            }
        }
        _ => {
            return Err(format!("Invalid message type: {}", msg_type).into());
        }
    }

    Ok(())
}
