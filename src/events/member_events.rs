use crate::config::AppState;
use crate::database::welcome_goodbye::{get_member_placeholders, WelcomeGoodbyeConfig};
use crate::logging::{error, warn};
use crate::utils::get_bot_channel_permissions;
use crate::utils::welcome_goodbye::{build_embed, replace_placeholders, EmbedConfig};
use serenity::{
    builder::CreateMessage,
    client::Context,
    model::{
        guild::Member,
        id::{ChannelId, GuildId},
        user::User,
    },
    prelude::{TypeMap, TypeMapKey},
};
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct Database;
impl TypeMapKey for Database {
    type Value = Arc<SqlitePool>;
}

pub struct AppStateKey;
impl TypeMapKey for AppStateKey {
    type Value = Arc<AppState>;
}

pub async fn member_addition(ctx: &Context, guild_id: &GuildId, new_member: &Member) {
    let data = ctx.data.read().await;
    let pool = match data.get::<Database>() {
        Some(pool) => pool,
        None => {
            error!("no db pool");
            return;
        }
    };

    let config = match WelcomeGoodbyeConfig::get_config(pool, &guild_id.to_string()).await {
        Ok(Some(config)) => config,
        Ok(None) => return, // No config found, nothing to do
        Err(e) => {
            error!("get welcome/goodbye config: {}", e);
            return;
        }
    };

    if !config.welcome_enabled || config.welcome_channel_id.is_none() {
        return;
    }

    let channel_id = match config.welcome_channel_id.as_ref().unwrap().parse::<u64>() {
        Ok(id) => ChannelId::new(id),
        Err(_) => {
            error!(
                "invalid welcome channel: {}",
                config.welcome_channel_id.unwrap()
            );
            return;
        }
    };

    let (guild_name, member_count) = match ctx.cache.guild(guild_id) {
        Some(guild) => (guild.name.clone(), guild.member_count),
        None => {
            error!("guild {} not in cache", guild_id);
            return;
        }
    };

    let placeholders = get_member_placeholders(
        &new_member.user,
        &guild_name,
        member_count,
        Some(new_member),
    );

    if let Err(e) = send_welcome_message(ctx, &channel_id, &config, &placeholders, &data).await {
        error!("send welcome message: {}", e);
    }
}

pub async fn member_removal(
    ctx: &Context,
    guild_id: &GuildId,
    user: &User,
    member_data_if_available: &Option<Member>,
) {
    let data = ctx.data.read().await;
    let pool = match data.get::<Database>() {
        Some(pool) => pool,
        None => {
            error!("no db pool");
            return;
        }
    };

    let config = match WelcomeGoodbyeConfig::get_config(pool, &guild_id.to_string()).await {
        Ok(Some(config)) => config,
        Ok(None) => return, // No config found, nothing to do
        Err(e) => {
            error!("get welcome/goodbye config: {}", e);
            return;
        }
    };

    if !config.goodbye_enabled || config.goodbye_channel_id.is_none() {
        return;
    }

    let channel_id = match config.goodbye_channel_id.as_ref().unwrap().parse::<u64>() {
        Ok(id) => ChannelId::new(id),
        Err(_) => {
            error!(
                "invalid goodbye channel: {}",
                config.goodbye_channel_id.unwrap()
            );
            return;
        }
    };

    let (guild_name, member_count) = match ctx.cache.guild(guild_id) {
        Some(guild) => (guild.name.clone(), guild.member_count),
        None => {
            error!("guild {} not in cache", guild_id);
            return;
        }
    };

    let placeholders = get_member_placeholders(
        user,
        &guild_name,
        member_count,
        member_data_if_available.as_ref(),
    );

    if let Err(e) = send_goodbye_message(ctx, &channel_id, &config, &placeholders, &data).await {
        error!("send goodbye message: {}", e);
    }
}

async fn send_welcome_message(
    ctx: &Context,
    channel_id: &ChannelId,
    config: &WelcomeGoodbyeConfig,
    placeholders: &std::collections::HashMap<String, String>,
    data: &tokio::sync::RwLockReadGuard<'_, TypeMap>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let channel = match ctx.http.get_channel(*channel_id).await {
        Ok(channel) => channel,
        Err(e) => {
            error!("get welcome channel: {}", e);
            return Err(Box::new(e));
        }
    };

    let guild_channel = match channel.guild() {
        Some(channel) => channel,
        None => {
            error!("welcome channel not guild");
            return Err("Channel is not a guild channel".into());
        }
    };

    // Check if bot has SEND_MESSAGES permission in the welcome channel
    let perms =
        match get_bot_channel_permissions(&ctx.http, guild_channel.guild_id, *channel_id).await {
            Some(p) => p,
            None => {
                error!("get channel permissions for welcome");
                return Err("Failed to get channel permissions".into());
            }
        };

    if !perms.permissions.send_messages() {
        warn!("no SEND_MESSAGES in welcome channel {}", channel_id);
        return Err("Bot lacks SEND_MESSAGES permission".into());
    }

    match config.welcome_message_type.as_str() {
        "embed" => {
            let default_color = data
                .get::<AppStateKey>()
                .map(|state| crate::utils::get_default_embed_color(state).0 as u64)
                .unwrap_or(0x5865F2);

            let embed_config = EmbedConfig {
                title: &config.welcome_embed_title,
                description: &config.welcome_embed_description,
                color: config.welcome_embed_color,
                footer: &config.welcome_embed_footer,
                thumbnail: &config.welcome_embed_thumbnail,
                image: &config.welcome_embed_image,
                timestamp: config.welcome_embed_timestamp,
                default_color,
            };

            let embed = build_embed(&embed_config, placeholders);
            let message = CreateMessage::new().embed(embed);
            guild_channel.send_message(&ctx.http, message).await?;
        }
        "text" => {
            if let Some(content) = &config.welcome_message_content {
                let processed_content = replace_placeholders(content, placeholders);
                if !processed_content.trim().is_empty() {
                    let message = CreateMessage::new().content(processed_content);
                    guild_channel.send_message(&ctx.http, message).await?;
                }
            }
        }
        _ => {
            error!("invalid welcome msg type: {}", config.welcome_message_type);
            return Err("Invalid message type".into());
        }
    }

    Ok(())
}

async fn send_goodbye_message(
    ctx: &Context,
    channel_id: &ChannelId,
    config: &WelcomeGoodbyeConfig,
    placeholders: &std::collections::HashMap<String, String>,
    data: &tokio::sync::RwLockReadGuard<'_, TypeMap>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let channel = match ctx.http.get_channel(*channel_id).await {
        Ok(channel) => channel,
        Err(e) => {
            error!("get goodbye channel: {}", e);
            return Err(Box::new(e));
        }
    };

    let guild_channel = match channel.guild() {
        Some(channel) => channel,
        None => {
            error!("goodbye channel not guild");
            return Err("Channel is not a guild channel".into());
        }
    };

    // Check if bot has SEND_MESSAGES permission in the goodbye channel
    let perms =
        match get_bot_channel_permissions(&ctx.http, guild_channel.guild_id, *channel_id).await {
            Some(p) => p,
            None => {
                error!("get channel permissions for goodbye");
                return Err("Failed to get channel permissions".into());
            }
        };

    if !perms.permissions.send_messages() {
        warn!("no SEND_MESSAGES in goodbye channel {}", channel_id);
        return Err("Bot lacks SEND_MESSAGES permission".into());
    }

    match config.goodbye_message_type.as_str() {
        "embed" => {
            let default_color = data
                .get::<AppStateKey>()
                .map(|state| crate::utils::get_default_embed_color(state).0 as u64)
                .unwrap_or(0x5865F2);

            let embed_config = EmbedConfig {
                title: &config.goodbye_embed_title,
                description: &config.goodbye_embed_description,
                color: config.goodbye_embed_color,
                footer: &config.goodbye_embed_footer,
                thumbnail: &config.goodbye_embed_thumbnail,
                image: &config.goodbye_embed_image,
                timestamp: config.goodbye_embed_timestamp,
                default_color,
            };

            let embed = build_embed(&embed_config, placeholders);
            let message = CreateMessage::new().embed(embed);
            guild_channel.send_message(&ctx.http, message).await?;
        }
        "text" => {
            if let Some(content) = &config.goodbye_message_content {
                let processed_content = replace_placeholders(content, placeholders);
                if !processed_content.trim().is_empty() {
                    let message = CreateMessage::new().content(processed_content);
                    guild_channel.send_message(&ctx.http, message).await?;
                }
            }
        }
        _ => {
            error!("invalid goodbye msg type: {}", config.goodbye_message_type);
            return Err("Invalid message type".into());
        }
    }

    Ok(())
}
