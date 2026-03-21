use crate::serenity;
use chrono::{Duration, Utc};
use clouder_core::config::AppState;
use clouder_core::database::selfroles::{SelfRoleConfig, SelfRoleCooldown};
use clouder_core::shared::check_interaction_expired;
use serenity::all::{CreateInteractionResponse, CreateInteractionResponseMessage, Mentionable};
use tracing::{error, info, warn};

pub async fn selfrole_message_delete(
    _ctx: &serenity::Context,
    _channel_id: &serenity::ChannelId,
    deleted_message_id: &serenity::MessageId,
    _guild_id: &Option<serenity::GuildId>,
    data: &AppState,
) {
    let message_id_str = deleted_message_id.to_string();

    match SelfRoleConfig::delete_by_message_id(&data.db, &message_id_str).await {
        Ok(true) => info!("selfrole config cleaned: {}", message_id_str),
        Ok(false) => {}
        Err(e) => error!("delete selfrole config: {}", e),
    }
}

async fn reply_ephemeral(
    interaction: &serenity::ComponentInteraction,
    ctx: &serenity::Context,
    content: &str,
) {
    if let Err(e) = interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .ephemeral(true),
            ),
        )
        .await
    {
        check_interaction_expired(&e);
    }
}

fn parse_selfrole_custom_id(custom_id: &str) -> Option<(i64, &str)> {
    let parts: Vec<&str> = custom_id.split('_').collect();
    if parts.len() != 3 {
        error!("invalid selfrole id: {}", custom_id);
        return None;
    }
    let config_id = match parts[1].parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            error!("invalid config_id: {}", parts[1]);
            return None;
        }
    };
    Some((config_id, parts[2]))
}

pub async fn handle_selfrole_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &AppState,
) {
    let (_, role_id) = match parse_selfrole_custom_id(&interaction.data.custom_id) {
        Some(parsed) => parsed,
        None => return,
    };

    let user_id = interaction.user.id.to_string();
    let guild_id = match interaction.guild_id {
        Some(id) => id.to_string(),
        None => {
            error!("selfrole outside guild");
            return;
        }
    };

    match SelfRoleCooldown::check_cooldown(&data.db, &user_id, role_id, &guild_id).await {
        Ok(true) => {
            reply_ephemeral(
                interaction,
                ctx,
                "You're doing that too quickly! Try again in a few seconds.",
            )
            .await;
            return;
        }
        Ok(false) => {}
        Err(e) => {
            error!("check cooldown: {}", e);
            reply_ephemeral(
                interaction,
                ctx,
                "an error occurred while processing your request. please try again.",
            )
            .await;
            return;
        }
    }

    let config = match SelfRoleConfig::get_by_message_id(
        &data.db,
        &interaction.message.id.to_string(),
    )
    .await
    {
        Ok(Some(config)) => config,
        Ok(None) => {
            error!("no selfrole config for message: {}", interaction.message.id);
            reply_ephemeral(
                interaction,
                ctx,
                "this self-role message is no longer valid.",
            )
            .await;
            return;
        }
        Err(e) => {
            error!("get selfrole config: {}", e);
            return;
        }
    };

    let guild_id_u64: u64 = match guild_id.parse() {
        Ok(id) => id,
        Err(_) => {
            error!("invalid guild_id: {}", guild_id);
            return;
        }
    };

    let role_id_u64: u64 = match role_id.parse() {
        Ok(id) => id,
        Err(_) => {
            error!("invalid role_id: {}", role_id);
            return;
        }
    };

    let member = match ctx
        .http
        .get_member(guild_id_u64.into(), interaction.user.id)
        .await
    {
        Ok(member) => member,
        Err(e) => {
            error!("get member {}: {}", interaction.user.id, e);
            reply_ephemeral(interaction, ctx, "failed to retrieve your member info.").await;
            return;
        }
    };

    let role = serenity::RoleId::new(role_id_u64);
    let has_role = member.roles.contains(&role);

    // Handle radio mode - remove other roles from this config first
    if config.selection_type == "radio" && !has_role {
        let config_roles = match config.get_roles(&data.db).await {
            Ok(roles) => roles,
            Err(e) => {
                error!("get config roles: {}", e);
                reply_ephemeral(
                    interaction,
                    ctx,
                    "an error occurred while processing your request.",
                )
                .await;
                return;
            }
        };

        for config_role in &config_roles {
            let config_role_id_u64: u64 = match config_role.role_id.parse() {
                Ok(id) => id,
                Err(_) => continue,
            };

            if member
                .roles
                .contains(&serenity::RoleId::new(config_role_id_u64))
                && let Err(e) = ctx
                    .http
                    .remove_member_role(
                        guild_id_u64.into(),
                        interaction.user.id,
                        serenity::RoleId::new(config_role_id_u64),
                        Some("Self-role radio mode"),
                    )
                    .await
            {
                warn!(
                    "remove role {} from {}: {}",
                    config_role_id_u64, interaction.user.id, e
                );
            }
        }
    }

    // Add or remove the role
    let (ok, message) = if has_role {
        match ctx
            .http
            .remove_member_role(
                guild_id_u64.into(),
                interaction.user.id,
                role,
                Some("Self-role removal"),
            )
            .await
        {
            Ok(_) => (true, format!("removed {}", role.mention())),
            Err(e) => {
                error!(
                    "remove role {} from {}: {}",
                    role_id_u64, interaction.user.id, e
                );
                (
                    false,
                    format!(
                        "failed to remove {}. i might not have permission or the role might not exist anymore.",
                        role.mention()
                    ),
                )
            }
        }
    } else {
        match ctx
            .http
            .add_member_role(
                guild_id_u64.into(),
                interaction.user.id,
                role,
                Some("Self-role assignment"),
            )
            .await
        {
            Ok(_) => (true, format!("added {}", role.mention())),
            Err(e) => {
                error!("add role {} to {}: {}", role_id_u64, interaction.user.id, e);
                (
                    false,
                    format!(
                        "failed to assign {}. the role may be managed by another bot, or is higher than my highest role in the server hierarchy.",
                        role.mention()
                    ),
                )
            }
        }
    };

    if ok {
        let expires_at = Utc::now() + Duration::seconds(5);
        if let Err(e) =
            SelfRoleCooldown::create(&data.db, &user_id, role_id, &guild_id, expires_at).await
        {
            error!("create cooldown: {}", e);
        }
    }

    reply_ephemeral(interaction, ctx, &message).await;
}
