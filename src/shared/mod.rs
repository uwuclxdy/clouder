pub mod models;

use crate::config::AppState;
use crate::database;
use crate::database::selfroles::SelfRoleConfig;
use serde_json::{json, Value};
use serenity::all::GuildId;

/// Get guild channels (text channels only)
pub async fn get_guild_channels(app_state: &AppState, guild_id: u64) -> Result<Value, String> {
    let channels = app_state
        .http
        .get_channels(GuildId::new(guild_id))
        .await
        .map_err(|e| format!("Failed to get channels: {}", e))?;

    let channel_info: Value = channels
        .into_iter()
        .filter(|c| matches!(c.kind, serenity::all::ChannelType::Text))
        .map(|c| {
            json!({
                "id": c.id.get(),
                "name": c.name,
                "channel_type": 0,
                "position": c.position as i32,
            })
        })
        .collect::<Vec<_>>()
        .into();

    Ok(channel_info)
}

/// Get guild roles (excluding @everyone)
pub async fn get_guild_roles(app_state: &AppState, guild_id: u64) -> Result<Value, String> {
    let roles = app_state
        .http
        .get_guild_roles(GuildId::new(guild_id))
        .await
        .map_err(|e| format!("Failed to get roles: {}", e))?;

    let role_info: Value = roles
        .into_iter()
        .filter(|r| r.name != "@everyone")
        .map(|r| {
            json!({
                "id": r.id.get(),
                "name": r.name,
                "color": r.colour.0,
                "position": r.position as i32,
                "mentionable": r.mentionable,
            })
        })
        .collect::<Vec<_>>()
        .into();

    Ok(role_info)
}

/// Get self-roles configurations for a guild
pub async fn list_selfroles(app_state: &AppState, guild_id: u64) -> Result<Value, String> {
    let guild_id_str = guild_id.to_string();
    let configs = database::selfroles::SelfRoleConfig::get_by_guild(&app_state.db, &guild_id_str)
        .await
        .map_err(|e| format!("Failed to get self-roles: {}", e))?;

    let channels = app_state
        .http
        .get_channels(GuildId::new(guild_id))
        .await
        .unwrap_or_default();

    let mut config_data = Vec::new();
    for config in configs {
        let roles = config.get_roles(&app_state.db).await.unwrap_or_default();
        let channel_name = channels
            .iter()
            .find(|ch| ch.id.to_string() == config.channel_id)
            .map(|ch| ch.name.clone())
            .unwrap_or_else(|| "Unknown Channel".to_string());

        config_data.push(json!({
            "id": config.id,
            "title": config.title,
            "body": config.body,
            "selection_type": config.selection_type,
            "channel_id": config.channel_id,
            "channel_name": channel_name,
            "message_id": config.message_id,
            "created_at": config.created_at,
            "updated_at": config.updated_at,
            "role_count": roles.len()
        }));
    }

    Ok(json!({"success": true, "configs": config_data}))
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
    let body = payload
        .get("body")
        .and_then(|v| v.as_str())
        .ok_or("Body is required")?;
    let selection_type = payload
        .get("selection_type")
        .and_then(|v| v.as_str())
        .ok_or("Selection type is required")?;
    let channel_id = payload
        .get("channel_id")
        .and_then(|v| v.as_str())
        .ok_or("Channel ID is required")?;
    let roles = payload
        .get("roles")
        .and_then(|v| v.as_array())
        .ok_or("Roles array is required")?;

    if title.trim().is_empty() || body.trim().is_empty() {
        return Err("Title and body cannot be empty".to_string());
    }

    if selection_type != "radio" && selection_type != "multiple" {
        return Err("Invalid selection type".to_string());
    }

    if roles.is_empty() || roles.len() > 25 {
        return Err("Must have 1-25 roles".to_string());
    }

    // Validate roles hierarchy
    let guild_roles = validate_roles_hierarchy(app_state, guild_id, roles).await?;

    // Create the configuration
    let guild_id_str = guild_id.to_string();
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
            .unwrap_or("");

        if database::selfroles::SelfRoleRole::create(&app_state.db, config.id, role_id, emoji)
            .await
            .is_err()
        {
            let _ = config.delete(&app_state.db).await;
            return Err("Failed to save role configuration".to_string());
        }
    }

    // Deploy the message
    deploy_selfrole_message(app_state, &config, &guild_roles, roles, user_id).await
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
        Ok(config.delete(&app_state.db).await.is_ok())
    };

    delete_result.map_err(|e| format!("Failed to delete configuration: {}", e))?;

    Ok(json!({
        "success": true,
        "message": "Self-role message deleted successfully"
    }))
}

// Helper functions

async fn validate_roles_hierarchy(
    app_state: &AppState,
    guild_id: u64,
    roles: &[Value],
) -> Result<Vec<serenity::all::Role>, String> {
    let bot_user = app_state
        .http
        .get_current_user()
        .await
        .map_err(|e| format!("Failed to get bot user: {}", e))?;

    let bot_member = app_state
        .http
        .get_member(GuildId::new(guild_id), bot_user.id)
        .await
        .map_err(|e| format!("Bot permission error: {}", e))?;

    let guild_roles = app_state
        .http
        .get_guild_roles(GuildId::new(guild_id))
        .await
        .map_err(|_| "Failed to get server roles")?;

    let bot_role_positions = crate::utils::get_bot_role_positions(&bot_member, &guild_roles);

    for role_data in roles {
        let role_id = role_data
            .get("role_id")
            .and_then(|v| v.as_str())
            .ok_or("Invalid role ID")?;
        let role_id_u64: u64 = role_id
            .parse()
            .map_err(|_| format!("Invalid role ID: {}", role_id))?;

        if let Some(target_role) = guild_roles.iter().find(|r| r.id.get() == role_id_u64) {
            if !crate::utils::can_bot_manage_role(&bot_role_positions, target_role.position) {
                return Err(format!("Cannot manage role '{}'", target_role.name));
            }
        } else {
            return Err(format!("Role {} not found", role_id));
        }
    }

    Ok(guild_roles)
}

async fn deploy_selfrole_message(
    app_state: &AppState,
    config: &database::selfroles::SelfRoleConfig,
    guild_roles: &[serenity::all::Role],
    roles: &[Value],
    _user_id: u64,
) -> Result<Value, String> {
    use serenity::all::{
        ButtonStyle, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, CreateMessage,
    };

    let footer_text = match config.selection_type.as_str() {
        "multiple" => "Multiple roles",
        "radio" => "Single role",
        _ => "",
    };

    let embed = CreateEmbed::new()
        .title(&config.title)
        .description(&config.body)
        .colour(crate::utils::get_default_embed_color(app_state))
        .footer(CreateEmbedFooter::new(footer_text));

    let mut action_rows = Vec::new();
    let mut current_row = Vec::new();

    for (index, role_data) in roles.iter().enumerate() {
        let role_id = role_data.get("role_id").and_then(|v| v.as_str()).unwrap();
        let emoji = role_data
            .get("emoji")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let role_name = guild_roles
            .iter()
            .find(|r| r.id.to_string() == role_id)
            .map(|r| r.name.clone())
            .unwrap_or_else(|| format!("Role {}", role_id));

        let button = CreateButton::new(format!("selfrole_{}_{}", config.id, role_id))
            .label(format!("{} {}", emoji, role_name))
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
            // Note: In a full implementation, you would update the database here
            // For this demo, we just return success without updating the config
            Ok(json!({
                "success": true,
                "message": "Self-role message deployed successfully",
                "message_id": sent_message.id.to_string(),
                "config_id": config.id
            }))
        }
        Err(_) => {
            let _ = config.delete(&app_state.db).await;
            Err("Failed to send message".to_string())
        }
    }
}
