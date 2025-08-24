use crate::config::AppState;
use crate::database::welcome_goodbye::{get_member_placeholders, replace_placeholders, WelcomeGoodbyeConfig};
use serenity::{
    builder::{CreateEmbed, CreateMessage},
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
            tracing::error!("Failed to get database pool from context");
            return;
        }
    };

    let config = match WelcomeGoodbyeConfig::get_config(pool, &guild_id.to_string()).await {
        Ok(Some(config)) => config,
        Ok(None) => return, // No config found, nothing to do
        Err(e) => {
            tracing::error!("Failed to get welcome/goodbye config: {}", e);
            return;
        }
    };

    if !config.welcome_enabled || config.welcome_channel_id.is_none() {
        return;
    }

    let channel_id = match config.welcome_channel_id.as_ref().unwrap().parse::<u64>() {
        Ok(id) => ChannelId::new(id),
        Err(_) => {
            tracing::error!("Invalid welcome channel ID: {}", config.welcome_channel_id.unwrap());
            return;
        }
    };

    let (guild_name, member_count) = match ctx.cache.guild(guild_id) {
        Some(guild) => (guild.name.clone(), guild.member_count),
        None => {
            tracing::error!("Guild not found in cache: {}", guild_id);
            return;
        }
    };

    let placeholders = get_member_placeholders(&new_member.user, &guild_name, member_count, Some(new_member));

    if let Err(e) = send_welcome_message(&ctx, &channel_id, &config, &placeholders, &data).await {
        tracing::error!("Failed to send welcome message: {}", e);
    }
}

pub async fn member_removal(ctx: &Context, guild_id: &GuildId, user: &User, member_data_if_available: &Option<Member>) {
    let data = ctx.data.read().await;
    let pool = match data.get::<Database>() {
        Some(pool) => pool,
        None => {
            tracing::error!("Failed to get database pool from context");
            return;
        }
    };

    let config = match WelcomeGoodbyeConfig::get_config(pool, &guild_id.to_string()).await {
        Ok(Some(config)) => config,
        Ok(None) => return, // No config found, nothing to do
        Err(e) => {
            tracing::error!("Failed to get welcome/goodbye config: {}", e);
            return;
        }
    };

    if !config.goodbye_enabled || config.goodbye_channel_id.is_none() {
        return;
    }

    let channel_id = match config.goodbye_channel_id.as_ref().unwrap().parse::<u64>() {
        Ok(id) => ChannelId::new(id),
        Err(_) => {
            tracing::error!("Invalid goodbye channel ID: {}", config.goodbye_channel_id.unwrap());
            return;
        }
    };

    let (guild_name, member_count) = match ctx.cache.guild(guild_id) {
        Some(guild) => (guild.name.clone(), guild.member_count),
        None => {
            tracing::error!("Guild not found in cache: {}", guild_id);
            return;
        }
    };

    let placeholders = get_member_placeholders(user, &guild_name, member_count, member_data_if_available.as_ref());

    if let Err(e) = send_goodbye_message(&ctx, &channel_id, &config, &placeholders, &data).await {
        tracing::error!("Failed to send goodbye message: {}", e);
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
            tracing::error!("Failed to get welcome channel: {}", e);
            return Err(Box::new(e));
        }
    };

    let guild_channel = match channel.guild() {
        Some(channel) => channel,
        None => {
            tracing::error!("Welcome channel is not a guild channel");
            return Err("Channel is not a guild channel".into());
        }
    };

    match config.welcome_message_type.as_str() {
        "embed" => {
            let mut embed = CreateEmbed::new();

            // Set title if provided
            if let Some(title) = &config.welcome_embed_title {
                if !title.trim().is_empty() {
                    embed = embed.title(replace_placeholders(title, placeholders));
                }
            }

            // Set description if provided
            if let Some(description) = &config.welcome_embed_description {
                if !description.trim().is_empty() {
                    embed = embed.description(replace_placeholders(description, placeholders));
                }
            }

            // Set color
            let color = config.welcome_embed_color
                .map(|c| c as u64)
                .unwrap_or_else(|| {
                    data.get::<AppStateKey>()
                        .map(|state| crate::utils::get_default_embed_color(state).0 as u64)
                        .unwrap_or(0x5865F2)
                });
            embed = embed.color(color);

            // Set footer if provided
            if let Some(footer) = &config.welcome_embed_footer {
                if !footer.trim().is_empty() {
                    embed = embed.footer(serenity::builder::CreateEmbedFooter::new(replace_placeholders(footer, placeholders)));
                }
            }

            // Set thumbnail if provided
            if let Some(thumbnail) = &config.welcome_embed_thumbnail {
                if !thumbnail.trim().is_empty() {
                    embed = embed.thumbnail(replace_placeholders(thumbnail, placeholders));
                }
            }

            // Set image if provided
            if let Some(image) = &config.welcome_embed_image {
                if !image.trim().is_empty() {
                    embed = embed.image(replace_placeholders(image, placeholders));
                }
            }

            // Set timestamp if enabled
            if config.welcome_embed_timestamp {
                embed = embed.timestamp(serenity::model::timestamp::Timestamp::now());
            }

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
            tracing::error!("Invalid welcome message type: {}", config.welcome_message_type);
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
            tracing::error!("Failed to get goodbye channel: {}", e);
            return Err(Box::new(e));
        }
    };

    let guild_channel = match channel.guild() {
        Some(channel) => channel,
        None => {
            tracing::error!("Goodbye channel is not a guild channel");
            return Err("Channel is not a guild channel".into());
        }
    };

    match config.goodbye_message_type.as_str() {
        "embed" => {
            let mut embed = CreateEmbed::new();

            // Set title if provided
            if let Some(title) = &config.goodbye_embed_title {
                if !title.trim().is_empty() {
                    embed = embed.title(replace_placeholders(title, placeholders));
                }
            }

            // Set description if provided
            if let Some(description) = &config.goodbye_embed_description {
                if !description.trim().is_empty() {
                    embed = embed.description(replace_placeholders(description, placeholders));
                }
            }

            // Set color
            let color = config.goodbye_embed_color
                .map(|c| c as u64)
                .unwrap_or_else(|| {
                    data.get::<AppStateKey>()
                        .map(|state| crate::utils::get_default_embed_color(state).0 as u64)
                        .unwrap_or(0x5865F2)
                });
            embed = embed.color(color);

            // Set footer if provided
            if let Some(footer) = &config.goodbye_embed_footer {
                if !footer.trim().is_empty() {
                    embed = embed.footer(serenity::builder::CreateEmbedFooter::new(replace_placeholders(footer, placeholders)));
                }
            }

            // Set thumbnail if provided
            if let Some(thumbnail) = &config.goodbye_embed_thumbnail {
                if !thumbnail.trim().is_empty() {
                    embed = embed.thumbnail(replace_placeholders(thumbnail, placeholders));
                }
            }

            // Set image if provided
            if let Some(image) = &config.goodbye_embed_image {
                if !image.trim().is_empty() {
                    embed = embed.image(replace_placeholders(image, placeholders));
                }
            }

            // Set timestamp if enabled
            if config.goodbye_embed_timestamp {
                embed = embed.timestamp(serenity::model::timestamp::Timestamp::now());
            }

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
            tracing::error!("Invalid goodbye message type: {}", config.goodbye_message_type);
            return Err("Invalid message type".into());
        }
    }

    Ok(())
}