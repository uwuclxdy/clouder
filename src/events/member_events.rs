use crate::logging::{error, warn};
use clouder_core::config::AppState;
use clouder_core::database::welcome_goodbye::{WelcomeGoodbyeConfig, get_member_placeholders};
use clouder_core::utils::welcome_goodbye::{EmbedConfig, build_embed, replace_placeholders};
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
            warn!("guild {} not in cache", guild_id);
            return;
        }
    };

    let placeholders = get_member_placeholders(
        &new_member.user,
        &guild_name,
        member_count,
        Some(new_member),
    );

    if let Err(e) = send_welcome_message(
        ctx,
        &channel_id,
        &config,
        &placeholders,
        &data,
        guild_id.get(),
    )
    .await
    {
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

    if let Err(e) = send_goodbye_message(
        ctx,
        &channel_id,
        &config,
        &placeholders,
        &data,
        guild_id.get(),
    )
    .await
    {
        error!("send goodbye message: {}", e);
    }
}

async fn send_welcome_message(
    ctx: &Context,
    channel_id: &ChannelId,
    config: &WelcomeGoodbyeConfig,
    placeholders: &std::collections::HashMap<String, String>,
    data: &tokio::sync::RwLockReadGuard<'_, TypeMap>,
    guild_id: u64,
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

    match config.welcome_message_type.as_str() {
        "embed" => {
            let state = data.get::<AppStateKey>().cloned().unwrap();
            let default_color = clouder_core::utils::get_embed_color(&state, Some(guild_id))
                .await
                .0 as u64;

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
    guild_id: u64,
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

    match config.goodbye_message_type.as_str() {
        "embed" => {
            let state = data.get::<AppStateKey>().cloned().unwrap();
            let default_color = clouder_core::utils::get_embed_color(&state, Some(guild_id))
                .await
                .0 as u64;

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
