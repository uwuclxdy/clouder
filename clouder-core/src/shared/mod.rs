pub mod models;

use crate::config::AppState;
use crate::database;
use crate::database::guild_cache::CachedGuild;
use crate::database::selfroles::{SelfRoleConfig, SelfRoleLabel};
use anyhow::Result;
use serde_json::{Value, json};
use serenity::all::{GuildId, Http, Permissions};
use tracing::{debug, error, warn};

const DISCORD_UNKNOWN_INTERACTION_ERROR_CODE: &str = "10062";

pub fn check_interaction_expired(error: &impl std::fmt::Display) {
    let error = error.to_string();
    let expired = error.contains(DISCORD_UNKNOWN_INTERACTION_ERROR_CODE)
        || error.contains("Unknown Interaction");
    if expired {
        debug!("interaction expired: {}", error);
    } else {
        error!("send cooldown response: {}", error);
    }
}

pub fn format_selfrole_button_label(emoji: &str, label: &str) -> String {
    let trimmed = emoji.trim();
    if trimmed.is_empty() {
        label.to_string()
    } else {
        format!("{} {}", trimmed, label)
    }
}

/// Get guild channels (text channels only)
pub async fn get_guild_channels(app_state: &AppState, guild_id: u64) -> Result<Value, String> {
    let channels = app_state
        .http
        .get_channels(GuildId::new(guild_id))
        .await
        .map_err(|e| format!("Failed to get channels: {}", e))?;

    let channel_list: Vec<Value> = channels
        .into_iter()
        .filter(|c| {
            matches!(
                c.kind,
                serenity::all::ChannelType::Text | serenity::all::ChannelType::News
            )
        })
        .map(|c| {
            json!({
                "id": c.id.to_string(),
                "name": c.name,
                "channel_type": match c.kind {
                    serenity::all::ChannelType::Text => 0,
                    serenity::all::ChannelType::News => 5,
                    _ => 0,
                },
                "position": c.position as i32,
            })
        })
        .collect();

    Ok(json!({ "channels": channel_list }))
}

/// Get guild roles (excluding @everyone)
pub async fn get_guild_roles(app_state: &AppState, guild_id: u64) -> Result<Value, String> {
    let roles = app_state
        .http
        .get_guild_roles(GuildId::new(guild_id))
        .await
        .map_err(|e| format!("Failed to get roles: {}", e))?;

    let role_list: Vec<Value> = roles
        .into_iter()
        .filter(|r| r.name != "@everyone" && !r.managed)
        .map(|r| {
            json!({
                "id": r.id.to_string(),
                "name": r.name,
                "color": r.colour.0,
                "position": r.position as i32,
                "mentionable": r.mentionable,
            })
        })
        .collect();

    Ok(json!({ "roles": role_list }))
}

pub async fn send_test_welcome_message(
    app_state: &AppState,
    guild_id: u64,
    msg_type: &str,
    user_id: &str,
) -> Result<(), String> {
    use crate::database::welcome_goodbye::WelcomeGoodbyeConfig;
    use crate::utils::welcome_goodbye::{EmbedConfig, build_embed, replace_placeholders};
    use serenity::all::{ChannelId, CreateMessage};
    use std::collections::HashMap;

    let config = WelcomeGoodbyeConfig::get_config(&app_state.db, &guild_id.to_string())
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or("No config found")?;

    let (channel_id, message_type, content, title, desc, color, footer, thumb, img, timestamp) =
        if msg_type == "welcome" {
            (
                config.welcome_channel_id,
                config.welcome_message_type,
                config.welcome_message_content,
                config.welcome_embed_title,
                config.welcome_embed_description,
                config.welcome_embed_color,
                config.welcome_embed_footer,
                config.welcome_embed_thumbnail,
                config.welcome_embed_image,
                config.welcome_embed_timestamp,
            )
        } else {
            (
                config.goodbye_channel_id,
                config.goodbye_message_type,
                config.goodbye_message_content,
                config.goodbye_embed_title,
                config.goodbye_embed_description,
                config.goodbye_embed_color,
                config.goodbye_embed_footer,
                config.goodbye_embed_thumbnail,
                config.goodbye_embed_image,
                config.goodbye_embed_timestamp,
            )
        };

    let channel_id = channel_id.ok_or("No channel configured")?;
    let channel: ChannelId = channel_id.parse().map_err(|_| "Invalid channel ID")?;

    let mut placeholders = HashMap::new();
    placeholders.insert("user".to_string(), format!("<@{}>", user_id));
    placeholders.insert("user_id".to_string(), user_id.to_string());
    placeholders.insert("username".to_string(), user_id.to_string());
    placeholders.insert("server".to_string(), guild_id.to_string());
    placeholders.insert("member_count".to_string(), "?".to_string());
    placeholders.insert("join_date".to_string(), "today".to_string());

    let mut msg = CreateMessage::new();
    if message_type == "embed" {
        let embed_config = EmbedConfig {
            title: &title,
            description: &desc,
            color,
            footer: &footer,
            thumbnail: &thumb,
            image: &img,
            timestamp,
            default_color: app_state.config.web.embed.default_color as u64,
        };
        msg = msg.embed(build_embed(&embed_config, &placeholders));
    } else if let Some(c) = content {
        msg = msg.content(replace_placeholders(&c, &placeholders));
    }

    app_state
        .http
        .send_message(channel, Vec::new(), &msg)
        .await
        .map_err(|e| format!("Failed to send: {}", e))?;
    Ok(())
}

/// Get self-roles configurations for a guild
pub async fn list_selfroles(app_state: &AppState, guild_id: u64) -> Result<Value, String> {
    let guild_id_str = guild_id.to_string();
    let configs = database::selfroles::SelfRoleConfig::get_by_guild(&app_state.db, &guild_id_str)
        .await
        .map_err(|e| format!("Failed to get self-roles: {}", e))?;

    let labels = SelfRoleLabel::get_all_for_guild(&app_state.db, &guild_id_str)
        .await
        .unwrap_or_default();
    let mut config_data = Vec::new();
    for config in configs {
        let roles = config.get_roles(&app_state.db).await.unwrap_or_default();

        let role_list: Vec<Value> = roles
            .iter()
            .map(|r| {
                let label = labels.get(&r.role_id).cloned().unwrap_or_default();
                json!({
                    "role_id": r.role_id,
                    "emoji": r.emoji,
                    "label": label,
                })
            })
            .collect();

        config_data.push(json!({
            "id": config.id,
            "guild_id": config.guild_id,
            "channel_id": config.channel_id,
            "message_id": config.message_id,
            "title": config.title,
            "description": config.body,
            "selection_type": config.selection_type,
            "roles": role_list,
            "created_at": config.created_at,
            "updated_at": config.updated_at,
        }));
    }

    Ok(json!({ "success": true, "configs": config_data }))
}

/// Create a new self-role configuration
pub async fn create_selfrole(
    app_state: &AppState,
    guild_id: u64,
    user_id: u64,
    payload: &Value,
) -> Result<Value, String> {
    // Validate the payload
    let title = payload
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or("Title is required")?;

    // Accept both 'body' and 'description' for backwards compatibility
    let body = payload
        .get("description")
        .or_else(|| payload.get("body"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Default to 'multiple' if not specified
    let selection_type = payload
        .get("selection_type")
        .and_then(|v| v.as_str())
        .unwrap_or("multiple");

    let channel_id = payload
        .get("channel_id")
        .and_then(|v| v.as_str())
        .ok_or("Channel ID is required")?;
    let roles = payload
        .get("roles")
        .and_then(|v| v.as_array())
        .ok_or("Roles array is required")?;

    if title.trim().is_empty() {
        return Err("Title cannot be empty".to_string());
    }

    if selection_type != "radio" && selection_type != "multiple" {
        return Err("Invalid selection type".to_string());
    }

    if roles.is_empty() || roles.len() > 25 {
        return Err("Must have 1-25 roles".to_string());
    }

    let guild_roles = app_state
        .http
        .get_guild_roles(GuildId::new(guild_id))
        .await
        .map_err(|_| "failed to get server roles")?;

    for role_data in roles {
        let role_id_str = role_data
            .get("role_id")
            .and_then(|v| v.as_str())
            .ok_or("invalid role id")?;
        let role_id_u64: u64 = role_id_str.parse().map_err(|_| "invalid role id")?;
        if let Some(role) = guild_roles.iter().find(|r| r.id.get() == role_id_u64)
            && role.managed
        {
            return Err(format!(
                "role '{}' is managed by another integration and cannot be assigned by me",
                role.name
            ));
        }
    }

    let guild_id_str = guild_id.to_string();
    // best-effort: cache discord role names
    let pairs_owned: Vec<(String, String)> = guild_roles
        .iter()
        .map(|r| (r.id.to_string(), r.name.clone()))
        .collect();
    let pairs_ref: Vec<(&str, &str)> = pairs_owned
        .iter()
        .map(|(id, name)| (id.as_str(), name.as_str()))
        .collect();
    let _ = SelfRoleLabel::upsert_many(&app_state.db, &guild_id_str, &pairs_ref).await;

    // Create the configuration
    let config = database::selfroles::SelfRoleConfig::create(
        &app_state.db,
        &guild_id_str,
        channel_id,
        title,
        body,
        selection_type,
    )
    .await
    .map_err(|e| format!("Failed to save configuration: {}", e))?;

    // Save roles
    for role_data in roles {
        let role_id = role_data
            .get("role_id")
            .and_then(|v| v.as_str())
            .ok_or("Role ID is required")?;
        let emoji = role_data
            .get("emoji")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if let Some(label) = role_data
            .get("label")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let _ = SelfRoleLabel::upsert(&app_state.db, &guild_id_str, role_id, label).await;
        }

        if database::selfroles::SelfRoleRole::create(&app_state.db, config.id, role_id, emoji)
            .await
            .is_err()
        {
            let _ = config.delete(&app_state.db).await;
            return Err("Failed to save role configuration".to_string());
        }
    }

    // Deploy the message
    deploy_selfrole_message(app_state, &config, guild_id, roles, user_id).await
}

/// Update an existing self-role configuration.
///
/// If the existing config has a stored message id and the channel stays the same,
/// the bot edits the existing message in place.
pub async fn update_selfrole(
    app_state: &AppState,
    guild_id: u64,
    config_id: i64,
    _user_id: u64,
    payload: &Value,
) -> Result<Value, String> {
    use serenity::all::{ChannelId, MessageId};
    use serenity::builder::EditMessage;

    let title = payload
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or("Title is required")?;

    // Accept both 'body' and 'description' for backwards compatibility
    let body = payload
        .get("description")
        .or_else(|| payload.get("body"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let selection_type = payload
        .get("selection_type")
        .and_then(|v| v.as_str())
        .unwrap_or("multiple");

    let channel_id = payload
        .get("channel_id")
        .and_then(|v| v.as_str())
        .ok_or("Channel ID is required")?;

    let roles = payload
        .get("roles")
        .and_then(|v| v.as_array())
        .ok_or("Roles array is required")?;

    if title.trim().is_empty() {
        return Err("Title cannot be empty".to_string());
    }

    if selection_type != "radio" && selection_type != "multiple" {
        return Err("Invalid selection type".to_string());
    }

    if roles.is_empty() || roles.len() > 25 {
        return Err("Must have 1-25 roles".to_string());
    }

    let guild_roles = app_state
        .http
        .get_guild_roles(GuildId::new(guild_id))
        .await
        .map_err(|_| "failed to get server roles")?;

    for role_data in roles {
        let role_id_str = role_data
            .get("role_id")
            .and_then(|v| v.as_str())
            .ok_or("invalid role id")?;
        let role_id_u64: u64 = role_id_str.parse().map_err(|_| "invalid role id")?;
        if let Some(role) = guild_roles.iter().find(|r| r.id.get() == role_id_u64)
            && role.managed
        {
            return Err(format!(
                "role '{}' is managed by another integration and cannot be assigned by me",
                role.name
            ));
        }
    }

    let pairs_owned: Vec<(String, String)> = guild_roles
        .iter()
        .map(|r| (r.id.to_string(), r.name.clone()))
        .collect();
    let pairs_ref: Vec<(&str, &str)> = pairs_owned
        .iter()
        .map(|(id, name)| (id.as_str(), name.as_str()))
        .collect();
    let _ = SelfRoleLabel::upsert_many(&app_state.db, &guild_id.to_string(), &pairs_ref).await;

    let mut config = SelfRoleConfig::get_by_id(&app_state.db, config_id)
        .await
        .map_err(|e| format!("Failed to get config: {}", e))?
        .ok_or("Configuration not found")?;

    if config.guild_id != guild_id.to_string() {
        return Err("Configuration not found".to_string());
    }

    let (embed, action_rows) = build_selfrole_embed_and_components(
        app_state,
        guild_id,
        config.id,
        title,
        body,
        selection_type,
        roles,
    )
    .await;

    let mut next_message_id = config.message_id.clone();

    let can_edit_in_place = config
        .message_id
        .as_deref()
        .is_some_and(|_| config.channel_id == channel_id);

    if can_edit_in_place {
        let channel_id_u64: u64 = config
            .channel_id
            .parse()
            .map_err(|_| "Invalid channel ID".to_string())?;
        let message_id_u64: u64 = config
            .message_id
            .as_deref()
            .unwrap()
            .parse()
            .map_err(|_| "Invalid message ID".to_string())?;

        let edit = EditMessage::new().embed(embed).components(action_rows);
        app_state
            .http
            .edit_message(
                ChannelId::new(channel_id_u64),
                MessageId::new(message_id_u64),
                &edit,
                Vec::new(),
            )
            .await
            .map_err(|e| format!("Failed to edit message: {}", e))?;
    } else {
        // Fallback: send a new message (e.g. missing message id or changed channel)
        if let Some(message_id) = &config.message_id {
            let channel_id_u64: u64 = config
                .channel_id
                .parse()
                .map_err(|_| "Invalid channel ID".to_string())?;
            let message_id_u64: u64 = message_id
                .parse()
                .map_err(|_| "Invalid message ID".to_string())?;
            let _ = app_state
                .http
                .delete_message(
                    channel_id_u64.into(),
                    message_id_u64.into(),
                    Some("Self-role updated"),
                )
                .await;
        }

        let channel_id_u64: u64 = channel_id
            .parse()
            .map_err(|_| "Invalid channel ID".to_string())?;

        let msg = serenity::all::CreateMessage::new()
            .embed(embed)
            .components(action_rows);

        let sent_message = app_state
            .http
            .send_message(channel_id_u64.into(), Vec::new(), &msg)
            .await
            .map_err(|e| format!("Failed to send message: {}", e))?;

        next_message_id = Some(sent_message.id.to_string());
    }

    // Persist db updates after discord succeeds
    if config.channel_id != channel_id {
        config
            .update_channel_id(&app_state.db, channel_id)
            .await
            .map_err(|e| format!("Failed to update channel: {}", e))?;
    }

    config
        .update(&app_state.db, title, body, selection_type)
        .await
        .map_err(|e| format!("Failed to update configuration: {}", e))?;

    database::selfroles::SelfRoleRole::delete_by_config_id(&app_state.db, config.id)
        .await
        .map_err(|e| format!("Failed to update roles: {}", e))?;

    for role_data in roles {
        let role_id = role_data
            .get("role_id")
            .and_then(|v| v.as_str())
            .ok_or("Role ID is required")?;
        let emoji = role_data
            .get("emoji")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if let Some(label) = role_data
            .get("label")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let _ =
                SelfRoleLabel::upsert(&app_state.db, &guild_id.to_string(), role_id, label).await;
        }

        if database::selfroles::SelfRoleRole::create(&app_state.db, config.id, role_id, emoji)
            .await
            .is_err()
        {
            return Err("Failed to save role configuration".to_string());
        }
    }

    if let Some(message_id) = next_message_id {
        if config.message_id.as_deref() != Some(message_id.as_str()) {
            config
                .update_message_id(&app_state.db, &message_id)
                .await
                .map_err(|e| format!("Failed to update message ID: {}", e))?;
        }

        Ok(json!({
            "success": true,
            "message": "Self-role updated successfully",
            "id": config.id,
            "message_id": message_id,
        }))
    } else {
        Err("Failed to update self-role".to_string())
    }
}

/// Delete a self-role configuration
pub async fn delete_selfrole(
    app_state: &AppState,
    guild_id: u64,
    config_id: i64,
) -> Result<Value, String> {
    let guild_id_str = guild_id.to_string();
    let configs = database::selfroles::SelfRoleConfig::get_by_guild(&app_state.db, &guild_id_str)
        .await
        .map_err(|e| format!("Failed to get self-roles: {}", e))?;

    let config = configs
        .into_iter()
        .find(|c| c.id == config_id)
        .ok_or("Configuration not found")?;

    // Delete the message if it exists
    if let Some(message_id) = &config.message_id {
        let channel_id_u64: u64 = config
            .channel_id
            .parse()
            .map_err(|_| "Invalid channel ID".to_string())?;
        let message_id_u64: u64 = message_id
            .parse()
            .map_err(|_| "Invalid message ID".to_string())?;
        let _ = app_state
            .http
            .delete_message(
                channel_id_u64.into(),
                message_id_u64.into(),
                Some("Self-role deleted"),
            )
            .await;
    }

    // Delete the configuration by message_id if available, otherwise by config_id
    let delete_result = if let Some(ref message_id) = config.message_id {
        SelfRoleConfig::delete_by_message_id(&app_state.db, message_id).await
    } else {
        config.delete(&app_state.db).await.map(|_| true)
    };

    delete_result.map_err(|e| format!("Failed to delete configuration: {}", e))?;

    Ok(json!({
        "success": true,
        "message": "Self-role message deleted successfully"
    }))
}

// Helper functions

async fn deploy_selfrole_message(
    app_state: &AppState,
    config: &database::selfroles::SelfRoleConfig,
    guild_id: u64,
    roles: &[Value],
    _user_id: u64,
) -> Result<Value, String> {
    use serenity::all::CreateMessage;

    let (embed, action_rows) = build_selfrole_embed_and_components(
        app_state,
        guild_id,
        config.id,
        &config.title,
        &config.body,
        &config.selection_type,
        roles,
    )
    .await;

    let message = CreateMessage::new().embed(embed).components(action_rows);
    let channel_id_u64: u64 = config
        .channel_id
        .parse()
        .map_err(|_| "Invalid channel ID".to_string())?;

    match app_state
        .http
        .send_message(channel_id_u64.into(), Vec::new(), &message)
        .await
    {
        Ok(sent_message) => {
            // Update the config with the message_id
            let message_id = sent_message.id.to_string();
            let _ = sqlx::query(
                "UPDATE selfrole_configs SET message_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
            )
            .bind(&message_id)
            .bind(config.id)
            .execute(app_state.db.as_ref())
            .await;

            Ok(json!({
                "success": true,
                "message": "Self-role message deployed successfully",
                "message_id": message_id,
                "id": config.id
            }))
        }
        Err(_) => {
            let _ = config.delete(&app_state.db).await;
            Err("Failed to send message".to_string())
        }
    }
}

async fn build_selfrole_embed_and_components(
    app_state: &AppState,
    guild_id: u64,
    config_id: i64,
    title: &str,
    body: &str,
    selection_type: &str,
    roles: &[Value],
) -> (
    serenity::all::CreateEmbed,
    Vec<serenity::all::CreateActionRow>,
) {
    use serenity::all::{
        ButtonStyle, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter,
    };

    let footer_text = match selection_type {
        "multiple" => "Multiple roles",
        "radio" => "Single role",
        _ => "",
    };

    let embed = CreateEmbed::new()
        .title(title)
        .description(body)
        .colour(crate::utils::get_embed_color(app_state, Some(guild_id)).await)
        .footer(CreateEmbedFooter::new(footer_text));

    let guild_id_str = guild_id.to_string();
    let mut action_rows = Vec::new();
    let mut current_row = Vec::new();

    for (index, role_data) in roles.iter().enumerate() {
        let role_id = role_data
            .get("role_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let emoji = role_data
            .get("emoji")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let label = if let Ok(Some(cached)) =
            SelfRoleLabel::get(&app_state.db, &guild_id_str, role_id).await
        {
            cached.name
        } else {
            // fallback: fetch from Discord and populate cache
            let name = app_state
                .http
                .get_guild_roles(GuildId::new(guild_id))
                .await
                .ok()
                .and_then(|rs| {
                    rs.into_iter()
                        .find(|r| r.id.to_string() == role_id)
                        .map(|r| r.name)
                })
                .unwrap_or_else(|| format!("role {}", role_id));
            let _ = SelfRoleLabel::upsert(&app_state.db, &guild_id_str, role_id, &name).await;
            name
        };

        let button_label = format_selfrole_button_label(emoji, &label);
        let button = CreateButton::new(format!("selfrole_{}_{}", config_id, role_id))
            .label(button_label)
            .style(ButtonStyle::Primary);

        current_row.push(button);

        if current_row.len() == 5 || index == roles.len() - 1 {
            action_rows.push(CreateActionRow::Buttons(current_row.clone()));
            current_row.clear();
            if action_rows.len() == 5 {
                break;
            }
        }
    }

    (embed, action_rows)
}

// Welcome/Goodbye functions

/// Get welcome/goodbye configuration for a guild
pub async fn get_welcome_goodbye_config(
    app_state: &AppState,
    guild_id: u64,
) -> Result<Value, String> {
    use crate::database::welcome_goodbye::WelcomeGoodbyeConfig;

    let guild_id_str = guild_id.to_string();
    let config = WelcomeGoodbyeConfig::get_config(&app_state.db, &guild_id_str)
        .await
        .map_err(|e| format!("Failed to get config: {}", e))?;

    let default_color = app_state.config.web.embed.default_color;
    if let Some(config) = config {
        Ok(json!({
            "success": true,
            "config": config,
            "default_color": default_color
        }))
    } else {
        Ok(json!({
            "success": true,
            "config": WelcomeGoodbyeConfig::default(),
            "default_color": default_color
        }))
    }
}

/// Update welcome/goodbye configuration for a guild
pub async fn update_welcome_goodbye_config(
    app_state: &AppState,
    guild_id: u64,
    payload: &Value,
) -> Result<Value, String> {
    use crate::database::welcome_goodbye::WelcomeGoodbyeConfig;

    let mut config = WelcomeGoodbyeConfig::get_config(&app_state.db, &guild_id.to_string())
        .await
        .map_err(|e| format!("Failed to get config: {}", e))?
        .unwrap_or_else(|| WelcomeGoodbyeConfig {
            guild_id: guild_id.to_string(),
            ..Default::default()
        });

    // Update fields from payload
    if let Some(v) = payload.get("welcome_enabled").and_then(|v| v.as_bool()) {
        config.welcome_enabled = v;
    }
    if let Some(v) = payload.get("goodbye_enabled").and_then(|v| v.as_bool()) {
        config.goodbye_enabled = v;
    }
    if let Some(v) = payload.get("welcome_channel_id").and_then(|v| v.as_str()) {
        config.welcome_channel_id = Some(v.to_string());
    }
    if let Some(v) = payload.get("goodbye_channel_id").and_then(|v| v.as_str()) {
        config.goodbye_channel_id = Some(v.to_string());
    }
    if let Some(v) = payload.get("welcome_message_type").and_then(|v| v.as_str()) {
        config.welcome_message_type = v.to_string();
    }
    if let Some(v) = payload.get("goodbye_message_type").and_then(|v| v.as_str()) {
        config.goodbye_message_type = v.to_string();
    }
    if let Some(v) = payload
        .get("welcome_message_content")
        .and_then(|v| v.as_str())
    {
        config.welcome_message_content = Some(v.to_string());
    }
    if let Some(v) = payload
        .get("goodbye_message_content")
        .and_then(|v| v.as_str())
    {
        config.goodbye_message_content = Some(v.to_string());
    }
    if let Some(v) = payload.get("welcome_embed_title").and_then(|v| v.as_str()) {
        config.welcome_embed_title = Some(v.to_string());
    }
    if let Some(v) = payload
        .get("welcome_embed_description")
        .and_then(|v| v.as_str())
    {
        config.welcome_embed_description = Some(v.to_string());
    }
    if let Some(v) = payload.get("welcome_embed_color").and_then(|v| v.as_i64()) {
        config.welcome_embed_color = Some(v as i32);
    }
    if let Some(v) = payload.get("welcome_embed_footer").and_then(|v| v.as_str()) {
        config.welcome_embed_footer = Some(v.to_string());
    }
    if let Some(v) = payload
        .get("welcome_embed_thumbnail")
        .and_then(|v| v.as_str())
    {
        config.welcome_embed_thumbnail = Some(v.to_string());
    }
    if let Some(v) = payload.get("welcome_embed_image").and_then(|v| v.as_str()) {
        config.welcome_embed_image = Some(v.to_string());
    }
    if let Some(v) = payload
        .get("welcome_embed_timestamp")
        .and_then(|v| v.as_bool())
    {
        config.welcome_embed_timestamp = v;
    }
    if let Some(v) = payload.get("goodbye_embed_title").and_then(|v| v.as_str()) {
        config.goodbye_embed_title = Some(v.to_string());
    }
    if let Some(v) = payload
        .get("goodbye_embed_description")
        .and_then(|v| v.as_str())
    {
        config.goodbye_embed_description = Some(v.to_string());
    }
    if let Some(v) = payload.get("goodbye_embed_color").and_then(|v| v.as_i64()) {
        config.goodbye_embed_color = Some(v as i32);
    }
    if let Some(v) = payload.get("goodbye_embed_footer").and_then(|v| v.as_str()) {
        config.goodbye_embed_footer = Some(v.to_string());
    }
    if let Some(v) = payload
        .get("goodbye_embed_thumbnail")
        .and_then(|v| v.as_str())
    {
        config.goodbye_embed_thumbnail = Some(v.to_string());
    }
    if let Some(v) = payload.get("goodbye_embed_image").and_then(|v| v.as_str()) {
        config.goodbye_embed_image = Some(v.to_string());
    }
    if let Some(v) = payload
        .get("goodbye_embed_timestamp")
        .and_then(|v| v.as_bool())
    {
        config.goodbye_embed_timestamp = v;
    }

    // validate URL fields
    for url in [
        &config.welcome_embed_thumbnail,
        &config.welcome_embed_image,
        &config.goodbye_embed_thumbnail,
        &config.goodbye_embed_image,
    ]
    .into_iter()
    .flatten()
    {
        if !crate::utils::is_valid_url(url) {
            return Err(format!(
                "invalid URL '{}': must start with http:// or https://",
                url
            ));
        }
    }

    // validate content lengths (Discord limits)
    let checks: &[(&Option<String>, usize, &str)] = &[
        (
            &config.welcome_message_content,
            2000,
            "welcome_message_content",
        ),
        (
            &config.goodbye_message_content,
            2000,
            "goodbye_message_content",
        ),
        (&config.welcome_embed_title, 256, "welcome_embed_title"),
        (&config.goodbye_embed_title, 256, "goodbye_embed_title"),
        (
            &config.welcome_embed_description,
            4096,
            "welcome_embed_description",
        ),
        (
            &config.goodbye_embed_description,
            4096,
            "goodbye_embed_description",
        ),
        (&config.welcome_embed_footer, 2048, "welcome_embed_footer"),
        (&config.goodbye_embed_footer, 2048, "goodbye_embed_footer"),
    ];
    for (field, max, name) in checks {
        validate_content_lengths(&[((*field).as_deref(), *max, *name)])?;
    }

    WelcomeGoodbyeConfig::upsert_config(&app_state.db, &config)
        .await
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(json!({
        "success": true,
        "message": "configuration saved successfully"
    }))
}

// MediaOnly functions

/// Get mediaonly configurations for a guild
pub async fn list_mediaonly_configs(app_state: &AppState, guild_id: u64) -> Result<Value, String> {
    use crate::database::mediaonly::MediaOnlyConfig;

    let guild_id_str = guild_id.to_string();
    let configs = MediaOnlyConfig::get_by_guild(&app_state.db, &guild_id_str)
        .await
        .map_err(|e| format!("Failed to get configs: {}", e))?;

    Ok(json!({
        "success": true,
        "configs": configs
    }))
}

/// Create or update a mediaonly configuration
pub async fn create_or_update_mediaonly_config(
    app_state: &AppState,
    guild_id: u64,
    channel_id: &str,
    payload: &Value,
) -> Result<Value, String> {
    use crate::database::mediaonly::MediaOnlyConfig;

    let allow_links = payload
        .get("allow_links")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let allow_attachments = payload
        .get("allow_attachments")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let allow_gifs = payload
        .get("allow_gifs")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let allow_stickers = payload
        .get("allow_stickers")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let guild_id_str = guild_id.to_string();
    MediaOnlyConfig::upsert_with_config(
        &app_state.db,
        &guild_id_str,
        channel_id,
        allow_links,
        allow_attachments,
        allow_gifs,
        allow_stickers,
    )
    .await
    .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(json!({
        "success": true,
        "message": "media-only channel configured successfully"
    }))
}

/// Delete a mediaonly configuration
pub async fn delete_mediaonly_config(
    app_state: &AppState,
    guild_id: u64,
    channel_id: &str,
) -> Result<Value, String> {
    use crate::database::mediaonly::MediaOnlyConfig;

    let guild_id_str = guild_id.to_string();
    MediaOnlyConfig::delete(&app_state.db, &guild_id_str, channel_id)
        .await
        .map_err(|e| format!("Failed to delete config: {}", e))?;

    Ok(json!({
        "success": true,
        "message": "media-only channel removed successfully"
    }))
}

pub async fn get_guild_config(app_state: &AppState, guild_id: u64) -> Result<Value, String> {
    use crate::database::guild_configs::GuildConfig;

    let config = GuildConfig::get_or_default(&app_state.db, &guild_id.to_string())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(json!({
        "timezone": config.timezone,
        "command_prefix": config.command_prefix,
        "embed_color": config.embed_color,
    }))
}

pub async fn update_guild_config(
    app_state: &AppState,
    guild_id: u64,
    payload: &Value,
) -> Result<Value, String> {
    use crate::database::guild_configs::GuildConfig;

    let guild_id_str = guild_id.to_string();
    let current = GuildConfig::get_or_default(&app_state.db, &guild_id_str)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let timezone = payload
        .get("timezone")
        .and_then(|v| v.as_str())
        .unwrap_or(&current.timezone);
    let command_prefix = payload
        .get("command_prefix")
        .and_then(|v| v.as_str())
        .unwrap_or(&current.command_prefix);
    let embed_color = payload.get("embed_color").and_then(|v| {
        let s = v.as_str()?;
        if s.is_empty() { None } else { Some(s) }
    });

    let updated = GuildConfig::upsert(
        &app_state.db,
        &guild_id_str,
        timezone,
        command_prefix,
        embed_color,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(json!({
        "timezone": updated.timezone,
        "command_prefix": updated.command_prefix,
        "embed_color": updated.embed_color,
    }))
}

pub async fn get_guild_about(app_state: &AppState, guild_id: u64) -> Result<Value, String> {
    use crate::database::guild_configs::GuildConfig;
    use crate::database::mediaonly::MediaOnlyConfig;
    use crate::database::welcome_goodbye::WelcomeGoodbyeConfig;

    let gid = GuildId::new(guild_id);
    let guild_id_str = guild_id.to_string();

    let (guild_result, channels_result, roles_result) = tokio::join!(
        app_state.http.get_guild_with_counts(gid),
        app_state.http.get_channels(gid),
        app_state.http.get_guild_roles(gid),
    );

    let guild = guild_result.map_err(|e| format!("failed to get guild: {}", e))?;
    let channels = channels_result.map_err(|e| format!("failed to get channels: {}", e))?;
    let roles = roles_result.map_err(|e| format!("failed to get roles: {}", e))?;

    let created_ms = (guild_id >> 22) + 1420070400000;
    let created_at = chrono::DateTime::from_timestamp_millis(created_ms as i64)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let text_channels = channels
        .iter()
        .filter(|c| {
            matches!(
                c.kind,
                serenity::all::ChannelType::Text | serenity::all::ChannelType::News
            )
        })
        .count();
    let voice_channels = channels
        .iter()
        .filter(|c| {
            matches!(
                c.kind,
                serenity::all::ChannelType::Voice | serenity::all::ChannelType::Stage
            )
        })
        .count();

    let role_count = roles.iter().filter(|r| r.name != "@everyone").count();

    let icon_url = guild.icon.as_ref().map(|h| {
        format!(
            "https://cdn.discordapp.com/icons/{}/{}.png?size=256",
            guild_id, h
        )
    });

    let boost_tier = match guild.premium_tier {
        serenity::all::PremiumTier::Tier1 => 1u8,
        serenity::all::PremiumTier::Tier2 => 2,
        serenity::all::PremiumTier::Tier3 => 3,
        _ => 0,
    };

    let owner_id = guild.owner_id;
    let (selfroles_result, mediaonly_result, wg_result, owner_result, config_result) = tokio::join!(
        database::selfroles::SelfRoleConfig::get_by_guild(&app_state.db, &guild_id_str),
        MediaOnlyConfig::get_by_guild(&app_state.db, &guild_id_str),
        WelcomeGoodbyeConfig::get_config(&app_state.db, &guild_id_str),
        app_state.http.get_user(owner_id),
        GuildConfig::get_or_default(&app_state.db, &guild_id_str),
    );

    let selfroles_count = selfroles_result.map(|v| v.len()).unwrap_or(0);
    let mediaonly_count = mediaonly_result.map(|v| v.len()).unwrap_or(0);
    let wg = wg_result.unwrap_or(None);
    let guild_config = config_result.unwrap_or_else(|_| GuildConfig {
        guild_id: guild_id_str.clone(),
        timezone: crate::database::guild_configs::DEFAULT_TIMEZONE.to_string(),
        command_prefix: crate::database::guild_configs::DEFAULT_COMMAND_PREFIX.to_string(),
        embed_color: None,
    });
    let (owner_name, owner_avatar) = match owner_result {
        Ok(u) => (
            u.global_name.unwrap_or_else(|| u.name.clone()),
            u.avatar.as_ref().map(|h| {
                format!(
                    "https://cdn.discordapp.com/avatars/{}/{}.png?size=64",
                    u.id, h
                )
            }),
        ),
        Err(_) => (owner_id.to_string(), None),
    };

    Ok(json!({
        "name": guild.name,
        "icon_url": icon_url,
        "description": guild.description,
        "created_at": created_at,
        "owner_id": owner_id.to_string(),
        "owner_name": owner_name,
        "owner_avatar": owner_avatar,
        "member_count": guild.approximate_member_count,
        "online_count": guild.approximate_presence_count,
        "total_channels": channels.len(),
        "text_channels": text_channels,
        "voice_channels": voice_channels,
        "role_count": role_count,
        "boost_tier": boost_tier,
        "boost_count": guild.premium_subscription_count.unwrap_or(0),
        "features": guild.features,
        "selfroles_panels": selfroles_count,
        "mediaonly_channels": mediaonly_count,
        "welcome_enabled": wg.as_ref().map(|c| c.welcome_enabled).unwrap_or(false),
        "goodbye_enabled": wg.as_ref().map(|c| c.goodbye_enabled).unwrap_or(false),
        "config_timezone": guild_config.timezone,
        "config_command_prefix": guild_config.command_prefix,
        "config_embed_color": guild_config.embed_color,
        "config_default_color": format!("#{:06X}", app_state.config.web.embed.default_color),
    }))
}

/// Fetches the user's and bot's guild lists in parallel, intersects them filtered by
/// management permissions, updates the DB cache, and returns `(guilds, updated)`.
pub async fn refresh_guild_cache(
    state: &AppState,
    user_id: &str,
    access_token: &str,
) -> Result<(Vec<models::GuildCacheEntry>, bool), String> {
    let (user_guilds, bot_guild_ids) = tokio::join!(
        fetch_discord_user_guilds(access_token),
        fetch_bot_guild_ids(&state.config.discord.token),
    );

    let mut mutual_guilds: Vec<models::GuildCacheEntry> = user_guilds
        .iter()
        .filter_map(|g| {
            let id = g["id"].as_str()?;
            let perms: u64 = g["permissions"].as_str()?.parse().ok()?;
            let perms_flags = Permissions::from_bits_truncate(perms);
            // include guilds where the user has any management-level permission
            let has_access = crate::utils::has_permission(perms_flags, Permissions::MANAGE_GUILD)
                || crate::utils::has_permission(perms_flags, Permissions::MANAGE_ROLES)
                || crate::utils::has_permission(perms_flags, Permissions::MANAGE_CHANNELS);
            if !has_access {
                return None;
            }
            if !bot_guild_ids.contains(id) {
                return None;
            }
            Some(models::GuildCacheEntry {
                id: id.to_string(),
                name: g["name"].as_str()?.to_string(),
                icon: g["icon"].as_str().map(String::from),
                permissions: perms,
            })
        })
        .collect();

    mutual_guilds.sort_by(|a, b| a.name.cmp(&b.name));

    let cached = CachedGuild::get_for_user(&state.db, user_id)
        .await
        .unwrap_or_default();

    let cached_ids: std::collections::HashSet<&str> =
        cached.iter().map(|g| g.guild_id.as_str()).collect();
    let new_ids: std::collections::HashSet<&str> =
        mutual_guilds.iter().map(|g| g.id.as_str()).collect();
    let updated = cached_ids != new_ids;

    let tuples: Vec<(String, String, Option<String>, i64)> = mutual_guilds
        .iter()
        .map(|g| {
            (
                g.id.clone(),
                g.name.clone(),
                g.icon.clone(),
                g.permissions as i64,
            )
        })
        .collect();

    if let Err(e) = CachedGuild::replace_for_user(&state.db, user_id, &tuples).await {
        warn!("failed to update guild cache for user {}: {}", user_id, e);
    }

    Ok((mutual_guilds, updated))
}

async fn fetch_discord_user_guilds(access_token: &str) -> Vec<Value> {
    let client = reqwest::Client::new();
    match client
        .get("https://discord.com/api/v10/users/@me/guilds?limit=200")
        .bearer_auth(access_token)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<Vec<Value>>().await.unwrap_or_default()
        }
        Ok(resp) => {
            warn!("discord user guilds: unexpected status {}", resp.status());
            vec![]
        }
        Err(e) => {
            warn!("failed to fetch user guilds: {}", e);
            vec![]
        }
    }
}

async fn fetch_bot_guild_ids(bot_token: &str) -> std::collections::HashSet<String> {
    let client = reqwest::Client::new();
    match client
        .get("https://discord.com/api/v10/users/@me/guilds?limit=200")
        .header("Authorization", format!("Bot {}", bot_token))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let guilds: Vec<Value> = resp.json().await.unwrap_or_default();
            guilds
                .iter()
                .filter_map(|g| g["id"].as_str().map(String::from))
                .collect()
        }
        Ok(resp) => {
            warn!("bot guild list: unexpected status {}", resp.status());
            std::collections::HashSet::new()
        }
        Err(e) => {
            warn!("failed to fetch bot guilds: {}", e);
            std::collections::HashSet::new()
        }
    }
}

// Uwufy functions

pub async fn list_uwufy_members(app_state: &AppState, guild_id: u64) -> Result<Value, String> {
    use crate::database::uwufy::UwufyToggle;

    let gid = GuildId::new(guild_id);
    let guild_id_str = guild_id.to_string();

    let mut all_members = Vec::new();
    let mut after: Option<u64> = None;
    loop {
        let members = app_state
            .http
            .get_guild_members(gid, Some(1000), after)
            .await
            .map_err(|e| format!("failed to get guild members: {}", e))?;

        if members.is_empty() {
            break;
        }
        after = members.last().map(|m| m.user.id.get());
        let is_last_page = members.len() < 1000;
        all_members.extend(members);
        if is_last_page {
            break;
        }
    }

    let enabled_ids = UwufyToggle::get_enabled_in_guild(&app_state.db, &guild_id_str)
        .await
        .unwrap_or_default();

    let member_list: Vec<Value> = all_members
        .into_iter()
        .filter(|m| !m.user.bot)
        .map(|m| {
            let uid = m.user.id.to_string();
            let display = m
                .nick
                .as_deref()
                .or(m.user.global_name.as_deref())
                .unwrap_or(&m.user.name);
            let avatar = m
                .user
                .avatar
                .as_ref()
                .map(|h| {
                    format!(
                        "https://cdn.discordapp.com/avatars/{}/{}.png?size=64",
                        m.user.id, h
                    )
                })
                .or_else(|| {
                    Some(format!(
                        "https://cdn.discordapp.com/embed/avatars/{}.png",
                        (m.user.id.get() >> 22) % 6
                    ))
                });
            json!({
                "user_id": uid,
                "username": m.user.name,
                "display_name": display,
                "avatar_url": avatar,
                "uwufy_enabled": enabled_ids.contains(&uid),
            })
        })
        .collect();

    Ok(json!({
        "success": true,
        "members": member_list,
        "enabled_count": enabled_ids.len(),
    }))
}

pub async fn toggle_uwufy_member(
    app_state: &AppState,
    guild_id: u64,
    user_id: &str,
    enabled: Option<bool>,
) -> Result<Value, String> {
    use crate::database::uwufy::UwufyToggle;

    let guild_id_str = guild_id.to_string();
    let new_state = match enabled {
        Some(state) => UwufyToggle::set_enabled(&app_state.db, &guild_id_str, user_id, state)
            .await
            .map_err(|e| format!("failed to toggle uwufy: {}", e))?,
        None => UwufyToggle::toggle(&app_state.db, &guild_id_str, user_id)
            .await
            .map_err(|e| format!("failed to toggle uwufy: {}", e))?,
    };

    Ok(json!({
        "success": true,
        "user_id": user_id,
        "enabled": new_state,
    }))
}

pub async fn disable_all_uwufy(app_state: &AppState, guild_id: u64) -> Result<Value, String> {
    use crate::database::uwufy::UwufyToggle;

    let guild_id_str = guild_id.to_string();
    let count = UwufyToggle::disable_all_in_guild(&app_state.db, &guild_id_str)
        .await
        .map_err(|e| format!("failed to disable all uwufy: {}", e))?;

    Ok(json!({
        "success": true,
        "disabled_count": count,
    }))
}

pub async fn send_dm_to_user(http: &Http, user_id: u64, content: &str) -> Result<()> {
    let channel = http
        .create_private_channel(&json!({ "recipient_id": user_id }))
        .await?;

    let channel_id = channel.id;
    for chunk in split_message_for_discord(content, 2000) {
        let msg = serenity::all::CreateMessage::new().content(&chunk);
        if let Err(e) = http.send_message(channel_id, Vec::new(), &msg).await {
            error!("failed to send dm chunk to {}: {}", user_id, e);
        }
    }

    Ok(())
}

fn split_message_for_discord(content: &str, max_length: usize) -> Vec<String> {
    if content.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut remaining = content;
    let max_length = max_length.max(1);

    while !remaining.is_empty() {
        let hard_end = get_hard_split_end(remaining, max_length);
        if hard_end == remaining.len() {
            chunks.push(remaining.to_string());
            break;
        }

        let mut end = find_preferred_split_end(remaining, hard_end).unwrap_or(hard_end);
        let split_breaks_token = split_breaks_markdown_token(remaining, end);
        let split_unbalances_markdown = !is_markdown_balanced(&remaining[..end]);

        if (split_breaks_token || split_unbalances_markdown)
            && let Some(safe_end) = find_balanced_split_before(remaining, end)
        {
            end = safe_end;
        }

        let (chunk, rest) = remaining.split_at(end);
        chunks.push(chunk.to_string());
        remaining = rest;
    }

    chunks
}

fn get_hard_split_end(content: &str, max_length: usize) -> usize {
    content
        .char_indices()
        .take_while(|(i, c)| *i + c.len_utf8() <= max_length)
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(content.len())
}

fn find_preferred_split_end(content: &str, hard_end: usize) -> Option<usize> {
    content[..hard_end]
        .char_indices()
        .filter_map(|(i, c)| c.is_whitespace().then_some(i))
        .filter(|&i| i > 0)
        .rfind(|&i| !split_breaks_markdown_token(content, i))
}

fn find_balanced_split_before(content: &str, from: usize) -> Option<usize> {
    let balanced_prefixes = collect_balanced_prefixes(content, from);

    content[..from]
        .char_indices()
        .map(|(i, _)| i)
        .rev()
        .find(|&i| i > 0 && !split_breaks_markdown_token(content, i) && balanced_prefixes[i])
}

#[derive(Clone, Copy, Default)]
struct MarkdownBalanceState {
    in_fence: bool,
    in_inline_code: bool,
    in_bold_asterisk: bool,
    in_bold_underscore: bool,
    in_strikethrough: bool,
}

impl MarkdownBalanceState {
    fn is_balanced(self) -> bool {
        !self.in_fence
            && !self.in_inline_code
            && !self.in_bold_asterisk
            && !self.in_bold_underscore
            && !self.in_strikethrough
    }
}

fn update_markdown_state(content: &str, mut i: usize, state: &mut MarkdownBalanceState) -> usize {
    if !is_escaped(content, i) {
        if content[i..].starts_with("```") {
            if !state.in_inline_code {
                state.in_fence = !state.in_fence;
            }
            i += 3;
            return i;
        }
        if !state.in_fence && content[i..].starts_with('`') {
            state.in_inline_code = !state.in_inline_code;
            i += 1;
            return i;
        }
        if !state.in_fence && !state.in_inline_code && content[i..].starts_with("**") {
            state.in_bold_asterisk = !state.in_bold_asterisk;
            i += 2;
            return i;
        }
        if !state.in_fence && !state.in_inline_code && content[i..].starts_with("__") {
            state.in_bold_underscore = !state.in_bold_underscore;
            i += 2;
            return i;
        }
        if !state.in_fence && !state.in_inline_code && content[i..].starts_with("~~") {
            state.in_strikethrough = !state.in_strikethrough;
            i += 2;
            return i;
        }
    }

    let ch_len = content[i..].chars().next().map(char::len_utf8).unwrap_or(1);
    i + ch_len
}

fn collect_balanced_prefixes(content: &str, from: usize) -> Vec<bool> {
    let mut balanced_prefixes = vec![false; from + 1];
    let mut state = MarkdownBalanceState::default();
    balanced_prefixes[0] = true;

    let mut i = 0;
    while i < from {
        i = update_markdown_state(content, i, &mut state);
        if i <= from {
            balanced_prefixes[i] = state.is_balanced();
        }
    }

    balanced_prefixes
}

fn is_markdown_balanced(content: &str) -> bool {
    let mut state = MarkdownBalanceState::default();

    let mut i = 0;
    while i < content.len() {
        i = update_markdown_state(content, i, &mut state);
    }

    state.is_balanced()
}

fn is_escaped(content: &str, marker_index: usize) -> bool {
    if marker_index == 0 {
        return false;
    }

    let mut slash_count = 0usize;
    for ch in content[..marker_index].chars().rev() {
        if ch == '\\' {
            slash_count += 1;
        } else {
            break;
        }
    }
    slash_count % 2 == 1
}

fn split_breaks_markdown_token(content: &str, split_at: usize) -> bool {
    const MARKERS: [&[u8]; 4] = [b"```", b"**", b"__", b"~~"];
    let content_bytes = content.as_bytes();

    MARKERS.iter().any(|marker| {
        let marker_len = marker.len();
        let start_min = split_at.saturating_sub(marker_len.saturating_sub(1));
        let start_max = split_at.min(content_bytes.len().saturating_sub(marker_len));
        if start_min > start_max {
            return false;
        }

        (start_min..=start_max).any(|start| {
            let end = start + marker_len;
            start < split_at && split_at < end && &content_bytes[start..end] == *marker
        })
    })
}

// Reminder functions

const MAX_CUSTOM_REMINDERS_PER_GUILD: i64 = 10;

fn validate_content_lengths(fields: &[(Option<&str>, usize, &str)]) -> Result<(), String> {
    for &(field, max_length, field_name) in fields {
        if let Some(value) = field
            && value.len() > max_length
        {
            return Err(format!("{} exceeds {} characters", field_name, max_length));
        }
    }

    Ok(())
}

/// Get all reminder configs for a guild, with their ping roles
pub async fn get_reminders_config(app_state: &AppState, guild_id: u64) -> Result<Value, String> {
    use crate::database::reminders::{
        CustomReminder, CustomReminderPingRole, ReminderConfig, ReminderPingRole,
    };

    let guild_id_str = guild_id.to_string();
    let configs = ReminderConfig::get_by_guild(&app_state.db, &guild_id_str)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let mut result = Vec::new();
    for cfg in configs {
        let roles = ReminderPingRole::get_by_config(&app_state.db, cfg.id)
            .await
            .unwrap_or_default();
        let role_ids: Vec<String> = roles.into_iter().map(|r| r.role_id).collect();
        result.push(json!({
            "id": cfg.id,
            "reminder_type": cfg.reminder_type.as_str(),
            "enabled": cfg.enabled,
            "channel_id": cfg.channel_id,
            "message_type": cfg.message_type,
            "message_content": cfg.message_content,
            "embed_title": cfg.embed_title,
            "embed_description": cfg.embed_description,
            "embed_color": cfg.embed_color,
            "wysi_morning_time": cfg.wysi_morning_time,
            "wysi_evening_time": cfg.wysi_evening_time,
            "timezone": cfg.timezone,
            "ping_roles": role_ids,
        }));
    }

    let custom = CustomReminder::get_by_guild(&app_state.db, &guild_id_str)
        .await
        .unwrap_or_default();
    let mut custom_result = Vec::new();
    for cr in custom {
        let roles = CustomReminderPingRole::get_by_reminder(&app_state.db, cr.id)
            .await
            .unwrap_or_default();
        let role_ids: Vec<String> = roles.into_iter().map(|r| r.role_id).collect();
        custom_result.push(json!({
            "id": cr.id, "name": cr.name, "enabled": cr.enabled,
            "channel_id": cr.channel_id, "schedule_time": cr.schedule_time,
            "schedule_days": cr.schedule_days, "timezone": cr.timezone,
            "message_type": cr.message_type, "message_content": cr.message_content,
            "embed_title": cr.embed_title, "embed_description": cr.embed_description,
            "embed_color": cr.embed_color, "ping_roles": role_ids,
        }));
    }

    Ok(json!({ "success": true, "configs": result, "custom_reminders": custom_result }))
}

/// Upsert a reminder config for a guild
#[allow(clippy::too_many_arguments)]
pub async fn upsert_reminder_config(
    app_state: &AppState,
    guild_id: u64,
    payload: &Value,
) -> Result<Value, String> {
    use crate::database::reminders::{ReminderConfig, ReminderPingRole, ReminderType};

    let reminder_type_str = payload
        .get("reminder_type")
        .and_then(|v| v.as_str())
        .ok_or("reminder_type required")?;

    let rtype = ReminderType::parse(reminder_type_str).ok_or("invalid reminder_type")?;
    if rtype == ReminderType::Custom {
        return Err("custom reminders not yet supported".to_string());
    }

    let guild_id_str = guild_id.to_string();
    let channel_id = payload
        .get("channel_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let message_type = payload
        .get("message_type")
        .and_then(|v| v.as_str())
        .unwrap_or("embed");
    let message_content = payload
        .get("message_content")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let embed_title = payload
        .get("embed_title")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let embed_description = payload
        .get("embed_description")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let embed_color = payload.get("embed_color").and_then(|v| v.as_i64());
    let wysi_morning = payload
        .get("wysi_morning_time")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let wysi_evening = payload
        .get("wysi_evening_time")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let timezone = payload
        .get("timezone")
        .and_then(|v| v.as_str())
        .unwrap_or("UTC");

    // validate wysi time format (HH:MM)
    if let Some(t) = wysi_morning
        && !crate::utils::is_valid_hhmm(t)
    {
        return Err(format!("invalid wysi_morning_time '{}', expected HH:MM", t));
    }
    if let Some(t) = wysi_evening
        && !crate::utils::is_valid_hhmm(t)
    {
        return Err(format!("invalid wysi_evening_time '{}', expected HH:MM", t));
    }

    validate_content_lengths(&[
        (message_content, 2000, "message_content"),
        (embed_title, 256, "embed_title"),
        (embed_description, 4096, "embed_description"),
    ])?;

    // validate timezone
    if timezone.parse::<chrono_tz::Tz>().is_err() {
        return Err(format!("invalid timezone: {}", timezone));
    }

    // Ensure guild_config exists before creating reminder_config (foreign key requirement)
    use crate::database::reminders::GuildConfig;
    if GuildConfig::get(&app_state.db, &guild_id_str)
        .await
        .map_err(|e| e.to_string())?
        .is_none()
    {
        GuildConfig::upsert(&app_state.db, &guild_id_str, "!", None, timezone)
            .await
            .map_err(|e| format!("Failed to create guild config: {}", e))?;
    }

    let config_id = ReminderConfig::upsert(
        &app_state.db,
        &guild_id_str,
        &rtype,
        channel_id,
        message_type,
        message_content,
        embed_title,
        embed_description,
        embed_color,
        wysi_morning,
        wysi_evening,
        timezone,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    // update enabled flag
    if let Some(enabled) = payload.get("enabled").and_then(|v| v.as_bool()) {
        ReminderConfig::set_enabled(&app_state.db, config_id, enabled)
            .await
            .map_err(|e| format!("DB error: {}", e))?;
    }

    // replace ping roles
    if let Some(roles) = payload.get("ping_roles").and_then(|v| v.as_array()) {
        let role_ids: Vec<String> = roles
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        ReminderPingRole::set_roles(&app_state.db, config_id, &role_ids)
            .await
            .map_err(|e| format!("DB error: {}", e))?;
    }

    Ok(json!({
        "success": true,
        "id": config_id,
        "message": "reminder config saved"
    }))
}

/// Update or create user reminder settings (timezone + dm enabled)
pub async fn update_user_reminder_settings(
    app_state: &AppState,
    user_id: &str,
    timezone: &str,
    dm_enabled: bool,
) -> Result<Value, String> {
    use crate::database::reminders::UserSettings;

    if timezone.parse::<chrono_tz::Tz>().is_err() {
        return Err(format!("invalid timezone: {}", timezone));
    }

    UserSettings::upsert(&app_state.db, user_id, timezone, dm_enabled)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(json!({ "success": true }))
}

/// Get current user reminder settings
pub async fn get_user_reminder_settings(
    app_state: &AppState,
    user_id: &str,
) -> Result<Value, String> {
    use crate::database::reminders::UserSettings;

    let settings = UserSettings::get(&app_state.db, user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    if let Some(s) = settings {
        Ok(json!({
            "success": true,
            "timezone": s.timezone,
            "dm_reminders_enabled": s.dm_reminders_enabled,
        }))
    } else {
        Ok(json!({
            "success": true,
            "timezone": "UTC",
            "dm_reminders_enabled": true,
        }))
    }
}

/// List a user's active reminder subscriptions with minimal details
pub async fn list_user_subscriptions(app_state: &AppState, user_id: &str) -> Result<Value, String> {
    use crate::database::reminders::{
        CustomReminder, CustomReminderSubscription, ReminderConfig, ReminderSubscription,
    };

    let subs = ReminderSubscription::get_by_user(&app_state.db, user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let mut arr = Vec::new();
    for sub in subs {
        if let Ok(Some(cfg)) = ReminderConfig::get_by_id(&app_state.db, sub.config_id).await {
            arr.push(json!({
                "subscription_id": sub.id,
                "config_id": cfg.id,
                "guild_id": cfg.guild_id,
                "reminder_type": cfg.reminder_type.as_str(),
            }));
        }
    }

    let custom_subs = CustomReminderSubscription::get_by_user(&app_state.db, user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let mut custom_arr = Vec::new();
    for sub in custom_subs {
        if let Ok(Some(cr)) = CustomReminder::get_by_id(&app_state.db, sub.reminder_id).await {
            custom_arr.push(json!({
                "subscription_id": sub.id,
                "reminder_id": cr.id,
                "guild_id": cr.guild_id,
                "name": cr.name,
            }));
        }
    }

    Ok(json!({ "success": true, "subscriptions": arr, "custom_subscriptions": custom_arr }))
}

/// Subscribe a user to a reminder config
pub async fn add_user_subscription(
    app_state: &AppState,
    user_id: &str,
    config_id: i64,
) -> Result<Value, String> {
    use crate::database::reminders::ReminderSubscription;
    ReminderSubscription::subscribe(&app_state.db, user_id, config_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    Ok(json!({ "success": true }))
}

/// Unsubscribe a user from a reminder config
pub async fn remove_user_subscription(
    app_state: &AppState,
    user_id: &str,
    config_id: i64,
) -> Result<Value, String> {
    use crate::database::reminders::ReminderSubscription;
    ReminderSubscription::unsubscribe(&app_state.db, user_id, config_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    Ok(json!({ "success": true }))
}

/// Delete subscription by its database id (used for manage page)
pub async fn remove_subscription_by_id(
    app_state: &AppState,
    subscription_id: i64,
) -> Result<Value, String> {
    use crate::database::reminders::ReminderSubscription;
    ReminderSubscription::delete_by_id(&app_state.db, subscription_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    Ok(json!({ "success": true }))
}

// Custom reminder functions

pub async fn get_custom_reminders(app_state: &AppState, guild_id: u64) -> Result<Value, String> {
    use crate::database::reminders::{CustomReminder, CustomReminderPingRole};

    let guild_id_str = guild_id.to_string();
    let reminders = CustomReminder::get_by_guild(&app_state.db, &guild_id_str)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let mut result = Vec::new();
    for cr in reminders {
        let roles = CustomReminderPingRole::get_by_reminder(&app_state.db, cr.id)
            .await
            .unwrap_or_default();
        let role_ids: Vec<String> = roles.into_iter().map(|r| r.role_id).collect();
        result.push(json!({
            "id": cr.id, "name": cr.name, "enabled": cr.enabled,
            "channel_id": cr.channel_id, "schedule_time": cr.schedule_time,
            "schedule_days": cr.schedule_days, "timezone": cr.timezone,
            "message_type": cr.message_type, "message_content": cr.message_content,
            "embed_title": cr.embed_title, "embed_description": cr.embed_description,
            "embed_color": cr.embed_color, "ping_roles": role_ids,
        }));
    }

    Ok(json!({ "success": true, "custom_reminders": result }))
}

pub async fn create_custom_reminder(
    app_state: &AppState,
    guild_id: u64,
    payload: &Value,
) -> Result<Value, String> {
    use crate::database::reminders::{CustomReminder, CustomReminderPingRole};

    let guild_id_str = guild_id.to_string();

    let count = CustomReminder::count_by_guild(&app_state.db, &guild_id_str)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    if count >= MAX_CUSTOM_REMINDERS_PER_GUILD {
        return Err(format!(
            "maximum of {} custom reminders per guild reached",
            MAX_CUSTOM_REMINDERS_PER_GUILD
        ));
    }

    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or("name is required")?;
    let channel_id = payload
        .get("channel_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let schedule_time = payload
        .get("schedule_time")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or("schedule_time is required")?;
    let schedule_days = payload
        .get("schedule_days")
        .and_then(|v| v.as_str())
        .unwrap_or("0,1,2,3,4,5,6");
    let timezone = payload
        .get("timezone")
        .and_then(|v| v.as_str())
        .unwrap_or("UTC");
    let message_type = payload
        .get("message_type")
        .and_then(|v| v.as_str())
        .unwrap_or("embed");
    let message_content = payload
        .get("message_content")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let embed_title = payload
        .get("embed_title")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let embed_description = payload
        .get("embed_description")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let embed_color = payload.get("embed_color").and_then(|v| v.as_i64());

    // validate schedule_time (HH:MM)
    if !crate::utils::is_valid_hhmm(schedule_time) {
        return Err("invalid schedule_time format, expected HH:MM".to_string());
    }

    // validate schedule_days (comma-separated 0-6)
    for part in schedule_days.split(',') {
        let day: u32 = part
            .trim()
            .parse()
            .map_err(|_| format!("invalid day in schedule_days: {}", part))?;
        if day > 6 {
            return Err(format!("schedule day out of range 0-6: {}", day));
        }
    }

    // validate timezone
    if timezone.parse::<chrono_tz::Tz>().is_err() {
        return Err(format!("invalid timezone: {}", timezone));
    }

    validate_content_lengths(&[
        (message_content, 2000, "message_content"),
        (embed_title, 256, "embed_title"),
        (embed_description, 4096, "embed_description"),
    ])?;

    // ensure guild_config exists (foreign key requirement)
    use crate::database::reminders::GuildConfig;
    if GuildConfig::get(&app_state.db, &guild_id_str)
        .await
        .map_err(|e| e.to_string())?
        .is_none()
    {
        GuildConfig::upsert(&app_state.db, &guild_id_str, "!", None, timezone)
            .await
            .map_err(|e| format!("Failed to create guild config: {}", e))?;
    }

    let config_id = CustomReminder::create(
        &app_state.db,
        &guild_id_str,
        name,
        channel_id,
        schedule_time,
        schedule_days,
        timezone,
        message_type,
        message_content,
        embed_title,
        embed_description,
        embed_color,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    if let Some(roles) = payload.get("ping_roles").and_then(|v| v.as_array()) {
        let role_ids: Vec<String> = roles
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        CustomReminderPingRole::set_roles(&app_state.db, config_id, &role_ids)
            .await
            .map_err(|e| format!("DB error: {}", e))?;
    }

    if let Some(enabled) = payload.get("enabled").and_then(|v| v.as_bool()) {
        CustomReminder::set_enabled(&app_state.db, config_id, enabled)
            .await
            .map_err(|e| format!("DB error: {}", e))?;
    }

    Ok(json!({ "success": true, "id": config_id }))
}

pub async fn update_custom_reminder(
    app_state: &AppState,
    guild_id: u64,
    reminder_id: i64,
    payload: &Value,
) -> Result<Value, String> {
    use crate::database::reminders::{CustomReminder, CustomReminderPingRole};

    let guild_id_str = guild_id.to_string();
    let existing = CustomReminder::get_by_id(&app_state.db, reminder_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or("custom reminder not found")?;

    if existing.guild_id != guild_id_str {
        return Err("custom reminder not found".to_string());
    }

    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(&existing.name);
    let channel_id = payload
        .get("channel_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .or(existing.channel_id.as_deref());
    let schedule_time = payload
        .get("schedule_time")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(&existing.schedule_time);
    let schedule_days = payload
        .get("schedule_days")
        .and_then(|v| v.as_str())
        .unwrap_or(&existing.schedule_days);
    let timezone = payload
        .get("timezone")
        .and_then(|v| v.as_str())
        .unwrap_or(&existing.timezone);
    let message_type = payload
        .get("message_type")
        .and_then(|v| v.as_str())
        .unwrap_or(&existing.message_type);
    let message_content = payload
        .get("message_content")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .or(existing.message_content.as_deref());
    let embed_title = payload
        .get("embed_title")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .or(existing.embed_title.as_deref());
    let embed_description = payload
        .get("embed_description")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .or(existing.embed_description.as_deref());
    let embed_color = payload
        .get("embed_color")
        .and_then(|v| v.as_i64())
        .or(existing.embed_color);

    // validate schedule_time (HH:MM)
    if !crate::utils::is_valid_hhmm(schedule_time) {
        return Err("invalid schedule_time format, expected HH:MM".to_string());
    }

    // validate schedule_days (comma-separated 0-6)
    for part in schedule_days.split(',') {
        let day: u32 = part
            .trim()
            .parse()
            .map_err(|_| format!("invalid day in schedule_days: {}", part))?;
        if day > 6 {
            return Err(format!("schedule day out of range 0-6: {}", day));
        }
    }

    // validate timezone
    if timezone.parse::<chrono_tz::Tz>().is_err() {
        return Err(format!("invalid timezone: {}", timezone));
    }

    validate_content_lengths(&[
        (message_content, 2000, "message_content"),
        (embed_title, 256, "embed_title"),
        (embed_description, 4096, "embed_description"),
    ])?;

    CustomReminder::update(
        &app_state.db,
        reminder_id,
        name,
        channel_id,
        schedule_time,
        schedule_days,
        timezone,
        message_type,
        message_content,
        embed_title,
        embed_description,
        embed_color,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    if let Some(roles) = payload.get("ping_roles").and_then(|v| v.as_array()) {
        let role_ids: Vec<String> = roles
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        CustomReminderPingRole::set_roles(&app_state.db, reminder_id, &role_ids)
            .await
            .map_err(|e| format!("DB error: {}", e))?;
    }

    if let Some(enabled) = payload.get("enabled").and_then(|v| v.as_bool()) {
        CustomReminder::set_enabled(&app_state.db, reminder_id, enabled)
            .await
            .map_err(|e| format!("DB error: {}", e))?;
    }

    Ok(json!({ "success": true }))
}

pub async fn delete_custom_reminder(
    app_state: &AppState,
    guild_id: u64,
    reminder_id: i64,
) -> Result<Value, String> {
    use crate::database::reminders::CustomReminder;

    let guild_id_str = guild_id.to_string();
    let existing = CustomReminder::get_by_id(&app_state.db, reminder_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or("custom reminder not found")?;

    if existing.guild_id != guild_id_str {
        return Err("custom reminder not found".to_string());
    }

    CustomReminder::delete(&app_state.db, reminder_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(json!({ "success": true }))
}

#[cfg(test)]
mod tests {
    use super::split_message_for_discord;

    #[test]
    fn splits_before_unclosed_markdown_when_possible() {
        let content = format!("{} **bold text** tail", "a".repeat(1990));
        let chunks = split_message_for_discord(&content, 2000);

        assert_eq!(chunks.len(), 2);
        assert!(!chunks[0].contains("**"));
        assert_eq!(chunks.concat(), content);
    }

    #[test]
    fn keeps_balanced_markdown_for_each_chunk_when_possible() {
        let content = format!("{} **bold text** {}", "a".repeat(1990), "b".repeat(30));
        let chunks = split_message_for_discord(&content, 2000);

        for chunk in &chunks {
            assert!(chunk.len() <= 2000);
            assert!(super::is_markdown_balanced(chunk), "chunk was: {chunk}");
        }
        assert_eq!(chunks.concat(), content);
    }

    #[test]
    fn falls_back_to_hard_limit_when_no_safe_split_exists() {
        let content = format!("```{}```", "x".repeat(2100));
        let chunks = split_message_for_discord(&content, 2000);

        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|chunk| chunk.len() <= 2000));
        assert_eq!(chunks.concat(), content);
    }

    #[test]
    fn respects_max_length_with_multibyte_chars() {
        let content = format!("{}🙂tail", "a".repeat(1999));
        let chunks = split_message_for_discord(&content, 2000);

        assert_eq!(chunks.len(), 2);
        assert!(chunks.iter().all(|chunk| chunk.len() <= 2000));
        assert_eq!(chunks.concat(), content);
    }

    #[test]
    fn adjusts_when_hard_split_would_cut_markdown_token() {
        let content = format!("{}{}", "a".repeat(1999), "```code```");
        let chunks = split_message_for_discord(&content, 2000);

        assert_eq!(chunks[0].len(), 1999);
        assert!(!super::split_breaks_markdown_token(
            &content,
            chunks[0].len()
        ));
        assert_eq!(chunks.concat(), content);
    }
}
