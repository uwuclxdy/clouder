use crate::config::AppState;
use crate::database::selfroles::{SelfRoleConfig, SelfRoleCooldown};
use crate::serenity;
use chrono::{Duration, Utc};
use serenity::all::{CreateInteractionResponse, CreateInteractionResponseMessage, Mentionable};

/// Handle message events - primarily for bot mention help responses
pub async fn on_message(
    ctx: &serenity::Context,
    message: &serenity::Message,
    data: &AppState,
) {
    if message.author.bot {
        return;
    }

    let current_user = match ctx.http.get_current_user().await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Failed to get current user: {}", e);
            return;
        }
    };

    if message.mentions.iter().any(|u| u.id == current_user.id) {
        if let Err(e) = send_help_as_message(ctx, message, data).await {
            tracing::error!("Failed to send help message on mention: {}", e);
        }
    }
}

async fn send_help_as_message(
    ctx: &serenity::Context,
    message: &serenity::Message,
    data: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let commands = crate::commands::help::get_all_commands();
    let embed = crate::commands::help::create_help_embed(&commands, data);

    message.channel_id.send_message(
        &ctx.http,
        serenity::CreateMessage::new()
            .embed(embed)
            .reference_message(message)
    ).await?;

    Ok(())
}

pub async fn handle_interaction_create(
    ctx: &serenity::Context,
    interaction: &serenity::Interaction,
    data: &AppState,
) {
    match interaction {
        serenity::Interaction::Component(component_interaction) => {
            handle_component_interaction(ctx, component_interaction, data).await;
        }
        _ => {}
    }
}

pub async fn handle_message_delete(
    _ctx: &serenity::Context,
    _channel_id: &serenity::ChannelId,
    deleted_message_id: &serenity::MessageId,
    _guild_id: &Option<serenity::GuildId>,
    data: &AppState,
) {
    let message_id_str = deleted_message_id.to_string();

    if let Ok(Some(config)) = SelfRoleConfig::get_by_message_id(&data.db, &message_id_str).await {
        tracing::info!("Self-role message deleted from Discord: {}, cleaning up database", message_id_str);
        if let Err(e) = config.delete(&data.db).await {
            tracing::error!("Failed to delete self-role config after Discord message deletion: {}", e);
        } else {
            tracing::info!("Successfully cleaned up self-role config for deleted message: {}", message_id_str);
        }
    }
}

async fn handle_component_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &AppState,
) {
    if interaction.data.custom_id.starts_with("selfrole_") {
        handle_selfrole_interaction(ctx, interaction, data).await;
    }
}

async fn handle_selfrole_interaction(ctx: &serenity::Context, interaction: &serenity::ComponentInteraction, data: &AppState, )
{
    let parts: Vec<&str> = interaction.data.custom_id.split('_').collect();
    if parts.len() != 3 {
        tracing::error!("Invalid selfrole custom_id format: {}", interaction.data.custom_id);
        return;
    }

    let _config_id: i64 = match parts[1].parse() {
        Ok(id) => id,
        Err(_) => {
            tracing::error!("Invalid config_id in custom_id: {}", parts[1]);
            return;
        }
    };

    let role_id = parts[2];
    let user_id = interaction.user.id.to_string();
    let guild_id = match interaction.guild_id {
        Some(id) => id.to_string(),
        None => {
            tracing::error!("Self-role interaction outside of guild");
            return;
        }
    };

    match SelfRoleCooldown::check_cooldown(&data.db, &user_id, role_id, &guild_id).await {
        Ok(true) => {
            if let Err(e) = interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("⏰ you're doing that too quickly! please wait a moment before trying again.")
                            .ephemeral(true),
                    ),
                )
                .await
            {
                tracing::error!("Failed to respond to cooldown interaction: {}", e);
            }
            return;
        }
        Ok(false) => {}
        Err(e) => {
            tracing::error!("Failed to check cooldown: {}", e);
            if let Err(e) = interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("❌ an error occurred while processing your request. please try again.")
                            .ephemeral(true),
                    ),
                )
                .await
            {
                tracing::error!("Failed to respond to error interaction: {}", e);
            }
            return;
        }
    }

    let config = match SelfRoleConfig::get_by_message_id(&data.db, &interaction.message.id.to_string()).await {
        Ok(Some(config)) => config,
        Ok(None) => {
            tracing::error!("Self-role config not found for message: {}", interaction.message.id);
            if let Err(e) = interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("❌ this self-role message is no longer valid.")
                            .ephemeral(true),
                    ),
                )
                .await
            {
                tracing::error!("Failed to respond to invalid config interaction: {}", e);
            }
            return;
        }
        Err(e) => {
            tracing::error!("Failed to get self-role config: {}", e);
            return;
        }
    };

    let guild_id_u64: u64 = match guild_id.parse() {
        Ok(id) => id,
        Err(_) => {
            tracing::error!("Invalid guild_id: {}", guild_id);
            return;
        }
    };

    let role_id_u64: u64 = match role_id.parse() {
        Ok(id) => id,
        Err(_) => {
            tracing::error!("Invalid role_id: {}", role_id);
            return;
        }
    };

    let member = match ctx.http.get_member(guild_id_u64.into(), interaction.user.id).await {
        Ok(member) => member,
        Err(e) => {
            tracing::error!("Failed to get member {}: {}", interaction.user.id, e);
            if let Err(e) = interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("❌ failed to retrieve your member info.")
                            .ephemeral(true),
                    ),
                )
                .await
            {
                tracing::error!("Failed to respond to member fetch error: {}", e);
            }
            return;
        }
    };

    let guild_roles = match ctx.http.get_guild_roles(guild_id_u64.into()).await {
        Ok(roles) => roles,
        Err(e) => {
            tracing::error!("Failed to get guild roles for {}: {}", guild_id, e);
            if let Err(e) = interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("❌ failed to retrieve server roles.")
                            .ephemeral(true),
                    ),
                )
                .await
            {
                tracing::error!("Failed to respond to guild roles fetch error: {}", e);
            }
            return;
        }
    };

    let bot_member = match crate::web::get_bot_member_info(&ctx.http, guild_id_u64.into()).await {
        Ok(member) => member,
        Err(e) => {
            tracing::error!("Failed to get bot member info for guild {}: {:?}", guild_id, e);
            if let Err(e) = interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("❌ bot permissions could not be verified.")
                            .ephemeral(true),
                    ),
                )
                .await
            {
                tracing::error!("Failed to respond to bot member fetch error: {}", e);
            }
            return;
        }
    };

    let bot_role_positions = crate::utils::get_bot_role_positions(&bot_member, &guild_roles);

    let target_role = match guild_roles.iter().find(|r| r.id.get() == role_id_u64) {
        Some(role) => role,
        None => {
            tracing::error!("Role {} not found in guild {}", role_id, guild_id);
            if let Err(e) = interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("❌ the requested role no longer exists.")
                            .ephemeral(true),
                    ),
                )
                .await
            {
                tracing::error!("Failed to respond to missing role error: {}", e);
            }
            return;
        }
    };

    if !crate::utils::can_bot_manage_role(&bot_role_positions, target_role.position) {
        tracing::warn!("Role hierarchy validation failed: bot positions {:?} vs target role '{}' position {}",
                      bot_role_positions, target_role.name, target_role.position);
        if let Err(e) = interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(format!("❌ cannot manage the role '{}' - it is higher than or equal to all of my roles in the hierarchy.", target_role.mention()))
                        .ephemeral(true),
                ),
            )
            .await
        {
            tracing::error!("Failed to respond to hierarchy validation error: {}", e);
        }
        return;
    }    let has_role = member.roles.contains(&serenity::RoleId::new(role_id_u64));

    // Handle radio mode - remove other roles from this config first
    if config.selection_type == "radio" && !has_role {
        let config_roles = match config.get_roles(&data.db).await {
            Ok(roles) => roles,
            Err(e) => {
                tracing::error!("Failed to get config roles: {}", e);
                if let Err(e) = interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("❌ an error occurred while processing your request.")
                                .ephemeral(true),
                        ),
                    )
                    .await
                {
                    tracing::error!("Failed to respond to error: {}", e);
                }
                return;
            }
        };

        // Remove all other roles from this config (only remove roles the bot can manage)
        for config_role in &config_roles {
            let config_role_id_u64: u64 = match config_role.role_id.parse() {
                Ok(id) => id,
                Err(_) => continue,
            };

            if member.roles.contains(&serenity::RoleId::new(config_role_id_u64)) {
                // Check if bot can manage this role before attempting removal
                if let Some(remove_role) = guild_roles.iter().find(|r| r.id.get() == config_role_id_u64) {
                    if crate::utils::can_bot_manage_role(&bot_role_positions, remove_role.position) {
                        if let Err(e) = ctx.http.remove_member_role(
                            guild_id_u64.into(),
                            interaction.user.id,
                            serenity::RoleId::new(config_role_id_u64),
                            Some("Self-role radio mode"),
                        ).await {
                            tracing::error!("Failed to remove role {} from user {}: {}", config_role_id_u64, interaction.user.id, e);
                        }
                    } else {
                        tracing::warn!("Cannot remove role '{}' due to hierarchy restrictions", remove_role.name);
                    }
                }
            }
        }
    }

    // Add or remove the role
    let (action, emoji, message) = if has_role {
        // Remove role
        match ctx.http.remove_member_role(
            guild_id_u64.into(),
            interaction.user.id,
            serenity::RoleId::new(role_id_u64),
            Some("Self-role removal"),
        ).await {
            Ok(_) => ("removed", "➖", format!("removed '{}'", target_role.mention())),
            Err(e) => {
                tracing::error!("Failed to remove role {} from user {}: {}", role_id_u64, interaction.user.id, e);
                ("error", "❌", format!("failed to remove the role '{}'. i might not have permission or the role might not exist anymore.", target_role.name))
            }
        }
    } else {
        // Add role
        match ctx.http.add_member_role(
            guild_id_u64.into(),
            interaction.user.id,
            serenity::RoleId::new(role_id_u64),
            Some("Self-role assignment"),
        ).await {
            Ok(_) => ("added", "✅", format!("added '{}'", target_role.mention())),
            Err(e) => {
                tracing::error!("Failed to add role {} to user {}: {}", role_id_u64, interaction.user.id, e);
                ("error", "❌", format!("failed to assign the role '{}'. i might not have permission, or the role might be higher than my role in the hierarchy.", target_role.name))
            }
        }
    };

    // Set cooldown if the role operation was successful
    if action != "error" {
        let expires_at = Utc::now() + Duration::seconds(5); // 5-second cooldown
        if let Err(e) = SelfRoleCooldown::create(&data.db, &user_id, role_id, &guild_id, expires_at).await {
            tracing::error!("Failed to create cooldown: {}", e);
        }
    }

    // Respond to the interaction
    if let Err(e) = interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("{} {}", emoji, message))
                    .ephemeral(true),
            ),
        )
        .await
    {
        tracing::error!("Failed to respond to self-role interaction: {}", e);
    }
}
