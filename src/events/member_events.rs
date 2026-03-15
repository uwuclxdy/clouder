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
use tracing::{error, warn};

pub struct Database;
impl TypeMapKey for Database {
    type Value = Arc<SqlitePool>;
}

pub struct AppStateKey;
impl TypeMapKey for AppStateKey {
    type Value = Arc<AppState>;
}

async fn fetch_config(
    data: &tokio::sync::RwLockReadGuard<'_, TypeMap>,
    guild_id: &GuildId,
) -> Option<WelcomeGoodbyeConfig> {
    let pool = match data.get::<Database>() {
        Some(pool) => pool,
        None => {
            error!("no db pool");
            return None;
        }
    };
    match WelcomeGoodbyeConfig::get_config(pool, &guild_id.to_string()).await {
        Ok(Some(config)) => Some(config),
        Ok(None) => None,
        Err(e) => {
            error!("get welcome/goodbye config: {}", e);
            None
        }
    }
}

async fn send_member_message(
    ctx: &Context,
    channel_id: &ChannelId,
    msg_type: &str,
    embed_cfg: EmbedConfig<'_>,
    msg_content: Option<&str>,
    placeholders: &std::collections::HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let channel = match ctx.http.get_channel(*channel_id).await {
        Ok(channel) => channel,
        Err(e) => {
            error!("get channel: {}", e);
            return Err(Box::new(e));
        }
    };
    let guild_channel = match channel.guild() {
        Some(channel) => channel,
        None => {
            error!("channel not guild");
            return Err("channel is not a guild channel".into());
        }
    };
    match msg_type {
        "embed" => {
            let embed = build_embed(&embed_cfg, placeholders);
            guild_channel
                .send_message(&ctx.http, CreateMessage::new().embed(embed))
                .await?;
        }
        "text" => {
            if let Some(content) = msg_content {
                let processed = replace_placeholders(content, placeholders);
                if !processed.trim().is_empty() {
                    guild_channel
                        .send_message(&ctx.http, CreateMessage::new().content(processed))
                        .await?;
                }
            }
        }
        _ => {
            error!("invalid message type: {}", msg_type);
            return Err("invalid message type".into());
        }
    }
    Ok(())
}

pub async fn member_addition(ctx: &Context, guild_id: &GuildId, new_member: &Member) {
    let data = ctx.data.read().await;
    let config = match fetch_config(&data, guild_id).await {
        Some(config) => config,
        None => return,
    };

    if !config.welcome_enabled || config.welcome_channel_id.is_none() {
        return;
    }

    let Some(ref channel_id_str) = config.welcome_channel_id else {
        return;
    };
    let channel_id = match channel_id_str.parse::<u64>() {
        Ok(id) => ChannelId::new(id),
        Err(_) => {
            error!("invalid welcome channel: {}", channel_id_str);
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

    let Some(state) = data.get::<AppStateKey>().cloned() else {
        error!("no app state");
        return;
    };
    let default_color = clouder_core::utils::get_embed_color(&state, Some(guild_id.get()))
        .await
        .0 as u64;
    let embed_cfg = EmbedConfig {
        title: &config.welcome_embed_title,
        description: &config.welcome_embed_description,
        color: config.welcome_embed_color,
        footer: &config.welcome_embed_footer,
        thumbnail: &config.welcome_embed_thumbnail,
        image: &config.welcome_embed_image,
        timestamp: config.welcome_embed_timestamp,
        default_color,
    };

    if let Err(e) = send_member_message(
        ctx,
        &channel_id,
        &config.welcome_message_type,
        embed_cfg,
        config.welcome_message_content.as_deref(),
        &placeholders,
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
    let config = match fetch_config(&data, guild_id).await {
        Some(config) => config,
        None => return,
    };

    if !config.goodbye_enabled || config.goodbye_channel_id.is_none() {
        return;
    }

    let Some(ref channel_id_str) = config.goodbye_channel_id else {
        return;
    };
    let channel_id = match channel_id_str.parse::<u64>() {
        Ok(id) => ChannelId::new(id),
        Err(_) => {
            error!("invalid goodbye channel: {}", channel_id_str);
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

    let Some(state) = data.get::<AppStateKey>().cloned() else {
        error!("no app state");
        return;
    };
    let default_color = clouder_core::utils::get_embed_color(&state, Some(guild_id.get()))
        .await
        .0 as u64;
    let embed_cfg = EmbedConfig {
        title: &config.goodbye_embed_title,
        description: &config.goodbye_embed_description,
        color: config.goodbye_embed_color,
        footer: &config.goodbye_embed_footer,
        thumbnail: &config.goodbye_embed_thumbnail,
        image: &config.goodbye_embed_image,
        timestamp: config.goodbye_embed_timestamp,
        default_color,
    };

    if let Err(e) = send_member_message(
        ctx,
        &channel_id,
        &config.goodbye_message_type,
        embed_cfg,
        config.goodbye_message_content.as_deref(),
        &placeholders,
    )
    .await
    {
        error!("send goodbye message: {}", e);
    }
}
