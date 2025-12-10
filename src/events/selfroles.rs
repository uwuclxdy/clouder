use crate::config::AppState;
use crate::database::selfroles::{SelfRoleConfig, SelfRoleCooldown};
use crate::logging::{error, info, warn};
use crate::serenity;
use chrono::{Duration, Utc};
use serenity::all::{CreateInteractionResponse, CreateInteractionResponseMessage, Mentionable};

pub async fn selfrole_message_delete(
    _ctx: &serenity::Context,
    _channel_id: &serenity::ChannelId,
    deleted_message_id: &serenity::MessageId,
    _guild_id: &Option<serenity::GuildId>,
    data: &AppState,
) {
    let message_id_str = deleted_message_id.to_string();

    if let Ok(Some(config)) = SelfRoleConfig::get_by_message_id(&data.db, &message_id_str).await {
        info!("selfrole message deleted: {}", message_id_str);
        if let Err(e) = config.delete(&data.db).await {
            error!("delete selfrole config: {}", e);
        } else {
            info!("selfrole config cleaned: {}", message_id_str);
        }
    }
}

pub async fn handle_selfrole_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &AppState,
) {
    let parts: Vec<&str> = interaction.data.custom_id.split('_').collect();
    if parts.len() != 3 {
        error!("invalid selfrole id: {}", interaction.data.custom_id);
        return;
    }

    let _config_id: i64 = match parts[1].parse() {
        Ok(id) => id,
        Err(_) => {
            error!("invalid config_id: {}", parts[1]);
            return;
        }
    };

    let role_id = parts[2];
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
            if let Err(e) = interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("You're doing that too quickly! Try again in a few seconds.")
                            .ephemeral(true),
                    ),
                )
                .await
            {
                error!("respond cooldown: {}", e);
            }
            return;
        }
        Ok(false) => {}
        Err(e) => {
            error!("check cooldown: {}", e);
            if let Err(e) = interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("an error occurred while processing your request. please try again.")
                            .ephemeral(true),
                    ),
                )
                .await
            {
                error!("respond error: {}", e);
            }
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
            if let Err(e) = interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("this self-role message is no longer valid.")
                            .ephemeral(true),
                    ),
                )
                .await
            {
                error!("respond invalid config: {}", e);
            }
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
            if let Err(e) = interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("failed to retrieve your member info.")
                            .ephemeral(true),
                    ),
                )
                .await
            {
                error!("respond member err: {}", e);
            }
            return;
        }
    };

    let guild_roles = match ctx.http.get_guild_roles(guild_id_u64.into()).await {
        Ok(roles) => roles,
        Err(e) => {
            error!("get guild roles for {}: {}", guild_id, e);
            if let Err(e) = interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("failed to retrieve server roles.")
                            .ephemeral(true),
                    ),
                )
                .await
            {
                error!("respond guild roles err: {}", e);
            }
            return;
        }
    };

    let bot_member = match crate::web::get_bot_member_info(&ctx.http, guild_id_u64.into()).await {
        Ok(member) => member,
        Err(e) => {
            error!("get bot member for {}: {:?}", guild_id, e);
            if let Err(e) = interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("bot permissions could not be verified.")
                            .ephemeral(true),
                    ),
                )
                .await
            {
                error!("respond bot member err: {}", e);
            }
            return;
        }
    };

    // Check if bot has MANAGE_ROLES permission by checking its roles
    let bot_has_manage_roles = bot_member.roles.iter().any(|role_id| {
        guild_roles
            .iter()
            .find(|r| r.id == *role_id)
            .is_some_and(|role| role.permissions.administrator() || role.permissions.manage_roles())
    });

    if !bot_has_manage_roles {
        warn!("no MANAGE_ROLES in guild {}", guild_id);
        if let Err(e) = interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("i don't have permission to manage roles in this server.")
                        .ephemeral(true),
                ),
            )
            .await
        {
            error!("respond permission err: {}", e);
        }
        return;
    }

    let bot_role_positions = crate::utils::get_bot_role_positions(&bot_member, &guild_roles);

    let target_role = match guild_roles.iter().find(|r| r.id.get() == role_id_u64) {
        Some(role) => role,
        None => {
            error!("role {} not found in {}", role_id, guild_id);
            if let Err(e) = interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("the requested role no longer exists.")
                            .ephemeral(true),
                    ),
                )
                .await
            {
                error!("respond missing role err: {}", e);
            }
            return;
        }
    };

    if !crate::utils::can_bot_manage_role(&bot_role_positions, target_role.position) {
        warn!(
            "role hierarchy: bot {:?} vs '{}' pos {}",
            bot_role_positions, target_role.name, target_role.position
        );
        if let Err(e) = interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(format!("cannot manage the role '{}' - it is higher than or equal to all of my roles in the hierarchy.", target_role.mention()))
                        .ephemeral(true),
                ),
            )
            .await
        {
            error!("respond hierarchy err: {}", e);
        }
        return;
    }
    let has_role = member.roles.contains(&serenity::RoleId::new(role_id_u64));

    // Handle radio mode - remove other roles from this config first
    if config.selection_type == "radio" && !has_role {
        let config_roles = match config.get_roles(&data.db).await {
            Ok(roles) => roles,
            Err(e) => {
                error!("get config roles: {}", e);
                if let Err(e) = interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("an error occurred while processing your request.")
                                .ephemeral(true),
                        ),
                    )
                    .await
                {
                    error!("respond error: {}", e);
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

            if member
                .roles
                .contains(&serenity::RoleId::new(config_role_id_u64))
            {
                // Check if bot can manage this role before attempting removal
                if let Some(remove_role) = guild_roles
                    .iter()
                    .find(|r| r.id.get() == config_role_id_u64)
                {
                    if crate::utils::can_bot_manage_role(&bot_role_positions, remove_role.position)
                    {
                        if let Err(e) = ctx
                            .http
                            .remove_member_role(
                                guild_id_u64.into(),
                                interaction.user.id,
                                serenity::RoleId::new(config_role_id_u64),
                                Some("Self-role radio mode"),
                            )
                            .await
                        {
                            error!(
                                "remove role {} from {}: {}",
                                config_role_id_u64, interaction.user.id, e
                            );
                        }
                    } else {
                        warn!("can't remove '{}' due to hierarchy", remove_role.name);
                    }
                }
            }
        }
    }

    // Add or remove the role
    let (action, emoji, message) = if has_role {
        // Remove role
        match ctx
            .http
            .remove_member_role(
                guild_id_u64.into(),
                interaction.user.id,
                serenity::RoleId::new(role_id_u64),
                Some("Self-role removal"),
            )
            .await
        {
            Ok(_) => ("removed", "", format!("removed {}", target_role.mention())),
            Err(e) => {
                error!(
                    "remove role {} from {}: {}",
                    role_id_u64, interaction.user.id, e
                );
                (
                    "error",
                    "",
                    format!(
                        "failed to remove the role '{}'. i might not have permission or the role might not exist anymore.",
                        target_role.name
                    ),
                )
            }
        }
    } else {
        // Add role
        match ctx
            .http
            .add_member_role(
                guild_id_u64.into(),
                interaction.user.id,
                serenity::RoleId::new(role_id_u64),
                Some("Self-role assignment"),
            )
            .await
        {
            Ok(_) => ("added", "", format!("added {}", target_role.mention())),
            Err(e) => {
                error!("add role {} to {}: {}", role_id_u64, interaction.user.id, e);
                (
                    "error",
                    "",
                    format!(
                        "failed to assign the role '{}'. i might not have permission, or the role might be higher than my role in the hierarchy.",
                        target_role.name
                    ),
                )
            }
        }
    };

    // Set cooldown if the role operation was successful
    if action != "error" {
        let expires_at = Utc::now() + Duration::seconds(5); // 5-second cooldown
        if let Err(e) =
            SelfRoleCooldown::create(&data.db, &user_id, role_id, &guild_id, expires_at).await
        {
            error!("create cooldown: {}", e);
        }
    }

    // Respond to the interaction
    let response_content = if emoji.is_empty() {
        message
    } else {
        format!("{} {}", emoji, message)
    };
    if let Err(e) = interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(response_content)
                    .ephemeral(true),
            ),
        )
        .await
    {
        error!("respond selfrole: {}", e);
    }
}
