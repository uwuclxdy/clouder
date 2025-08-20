use crate::config::AppState;
use anyhow::Result;
use poise::serenity_prelude as serenity;
use serenity::{CreateEmbed, Color, CreateEmbedFooter};
use std::time::SystemTime;
use sysinfo::System;
use lazy_static::lazy_static;

lazy_static! {
pub static ref BOT_START_TIME: SystemTime = SystemTime::now();
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

#[poise::command(slash_command, subcommands("bot", "server", "user"))]
pub async fn about(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn bot(ctx: Context<'_>) -> Result<(), Error> {
    show_comprehensive_bot_info(ctx).await?;
    Ok(())
}

async fn show_comprehensive_bot_info(ctx: Context<'_>) -> Result<(), Error> {
    let uptime = BOT_START_TIME.elapsed().unwrap_or_default();
    let uptime_str = format_duration(uptime.as_secs());

    let bot_version = env!("CARGO_PKG_VERSION");

    // Simple latency - just measure ping to Discord
    let start = std::time::Instant::now();
    let _ = ctx.http().get_current_user().await;
    let latency = start.elapsed();
    let latency_str = format!("{}ms", latency.as_millis());

    // System information
    let mut sys = System::new_all();
    sys.refresh_all();

    // Memory information
    let total_memory = sys.total_memory() / 1024 / 1024; // Convert to MB
    let used_memory = sys.used_memory() / 1024 / 1024;
    let memory_usage = format!("{:.1}% ({} MB / {} MB)",
        (used_memory as f64 / total_memory as f64) * 100.0,
        used_memory,
        total_memory
    );

    // CPU information
    let cpu_usage = sys.global_cpu_usage();
    let cpu_count = sys.cpus().len();
    let cpu_info = format!("{:.1}% ({} cores)", cpu_usage, cpu_count);

    // System information
    let os_info = format!("{} {}",
        System::name().unwrap_or_else(|| "Unknown".to_string()),
        System::os_version().unwrap_or_else(|| "Unknown".to_string())
    );

    // Database statistics
    let db_stats = get_database_stats(ctx).await;

    // Additional bot information
    let guild_count = ctx.cache().guild_count();
    let cached_users = ctx.cache().user_count();

    // Process information
    let process_info = format!("PID: {}", std::process::id());

    let embed = CreateEmbed::new()
        .title("🤖 Clouder Bot - Comprehensive Information")
        .color(Color::BLITZ_BLUE)
        .field("📊 Bot Stats", format!("Version: {}\nUptime: {}\nLatency: {}", bot_version, uptime_str, latency_str), true)
        .field("🌐 Discord Stats", format!("Guilds: {}\nCached Users: {}", guild_count, cached_users), true)
        .field("💾 System Resources", format!("Memory: {}\nCPU: {}", memory_usage, cpu_info), true)
        .field("🖥️ Environment", format!("{}\n{}", os_info, process_info), true)
        .field("🗄️ Database Stats", db_stats, true)
        .field("👨‍💻 Made by", "uwuclxdy", true)
        .footer(CreateEmbedFooter::new("Clouder Discord Bot • All systems operational"));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

async fn get_database_stats(ctx: Context<'_>) -> String {
    let db = &ctx.data().db;

    // Get selfrole configurations count
    let selfrole_configs = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM selfrole_configs")
        .fetch_one(db.as_ref())
        .await.unwrap_or_else(|_| 0);

    // Get selfrole roles count
    let selfrole_roles = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM selfrole_roles")
        .fetch_one(db.as_ref())
        .await.unwrap_or_else(|_| 0);

    // Get active cooldowns count
    let active_cooldowns = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM selfrole_cooldowns WHERE expires_at > datetime('now')")
        .fetch_one(db.as_ref())
        .await.unwrap_or_else(|_| 0);

    // Get total guilds in database
    let db_guilds = sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT guild_id) FROM selfrole_configs")
        .fetch_one(db.as_ref())
        .await.unwrap_or_else(|_| 0);

    format!("Configs: {}\nRoles: {}\nCooldowns: {}\nDB Guilds: {}",
        selfrole_configs, selfrole_roles, active_cooldowns, db_guilds)
}

#[poise::command(slash_command)]
pub async fn server(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.send(poise::CreateReply::default()
                .content("❌ This command can only be used in a server!")
                .ephemeral(true))
                .await?;
            return Ok(());
        }
    };

    // Get guild information from the API
    let full_guild = match ctx.http().get_guild(guild_id).await {
        Ok(guild) => guild,
        Err(_) => {
            ctx.send(poise::CreateReply::default()
                .content("❌ Failed to fetch server information!")
                .ephemeral(true))
                .await?;
            return Ok(());
        }
    };

    let member_count = match full_guild.approximate_member_count {
        Some(count) => count,
        None => {
            // Try to get from cache or fallback to 0
            match ctx.guild() {
                Some(guild) => guild.member_count,
                None => 0,
            }
        }
    };

    let created_at = guild_id.created_at();
    let created_timestamp = format!("<t:{}:F>", created_at.timestamp());

    // Channel counts
    let channels = if let Ok(channels) = full_guild.channels(&ctx.http()).await {
        channels
    } else {
        std::collections::HashMap::new()
    };

    let text_channels = channels.values().filter(|c| matches!(c.kind, serenity::ChannelType::Text)).count();
    let voice_channels = channels.values().filter(|c| matches!(c.kind, serenity::ChannelType::Voice)).count();
    let total_channels = channels.len();

    // Role count
    let role_count = full_guild.roles.len();

    // Boost information
    let boost_level = match full_guild.premium_tier {
        serenity::PremiumTier::Tier0 => 0,
        serenity::PremiumTier::Tier1 => 1,
        serenity::PremiumTier::Tier2 => 2,
        serenity::PremiumTier::Tier3 => 3,
        _ => 0,
    };
    let boost_count = full_guild.premium_subscription_count.unwrap_or(0);

    // Owner information
    let owner = match full_guild.owner_id.to_user(&ctx.http()).await {
        Ok(user) => format!("{} ({})", user.tag(), user.id),
        Err(_) => format!("Unknown ({})", full_guild.owner_id),
    };

    // Server features
    let features = if full_guild.features.is_empty() {
        "None".to_string()
    } else {
        full_guild.features.iter()
            .map(|f| format!("{:?}", f))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let mut embed = CreateEmbed::new()
        .title(&format!("📊 {} Server Information", full_guild.name))
        .color(Color::PURPLE)
        .field("👥 Members", member_count.to_string(), true)
        .field("💬 Channels", format!("{} ({} text, {} voice)", total_channels, text_channels, voice_channels), true)
        .field("🎭 Roles", role_count.to_string(), true)
        .field("👑 Owner", owner, false)
        .field("📅 Created", created_timestamp, true)
        .field("🚀 Boost Level", format!("Level {} ({} boosts)", boost_level, boost_count), true)
        .field("🏷️ Server ID", guild_id.to_string(), true);

    if features != "None" {
        embed = embed.field("✨ Features", features, false);
    }

    // Add server icon if available
    if let Some(icon_url) = full_guild.icon_url() {
        embed = embed.thumbnail(icon_url);
    }

    // Add server banner if available
    if let Some(banner_url) = full_guild.banner_url() {
        embed = embed.image(banner_url);
    }

    embed = embed.footer(CreateEmbedFooter::new("Server Statistics"));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn user(
    ctx: Context<'_>,
    #[description = "User to get information about"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user = user.unwrap_or_else(|| ctx.author().clone());

    // Get member information if in a guild
    let member_info = if let Some(guild_id) = ctx.guild_id() {
        match guild_id.member(&ctx.http(), target_user.id).await {
            Ok(member) => Some(member),
            Err(_) => None,
        }
    } else {
        None
    };

    let created_at = target_user.id.created_at();
    let account_age = format!("<t:{}:F> (<t:{}:R>)", created_at.timestamp(), created_at.timestamp());

    let mut embed = CreateEmbed::new()
        .title(&format!("👤 {} User Information", target_user.tag()))
        .color(Color::BLUE)
        .field("🏷️ User ID", target_user.id.to_string(), true)
        .field("📅 Account Created", account_age, false);

    // Add bot indicator
    if target_user.bot {
        embed = embed.field("🤖 Bot", "Yes", true);
    }

    // Add member-specific information if available
    if let Some(member) = member_info {
        if let Some(joined_at) = member.joined_at {
            let join_info = format!("<t:{}:F> (<t:{}:R>)", joined_at.timestamp(), joined_at.timestamp());
            embed = embed.field("📥 Joined Server", join_info, false);
        }

        // Display name (nickname or username)
        let display_name = member.display_name().to_string();
        if display_name != target_user.name {
            embed = embed.field("📝 Nickname", display_name, true);
        }

        // Roles (excluding @everyone)
        let roles: Vec<String> = member.roles
            .iter()
            .filter_map(|role_id| {
                if let Some(guild) = ctx.guild() {
                    guild.roles.get(role_id).map(|role| format!("<@&{}>", role.id))
                } else {
                    None
                }
            })
            .collect();

        if !roles.is_empty() {
            let roles_text = if roles.len() > 10 {
                format!("{} and {} more...", roles[..10].join(" "), roles.len() - 10)
            } else {
                roles.join(" ")
            };
            embed = embed.field(&format!("🎭 Roles ({})", roles.len()), roles_text, false);
        }

        // Boost status
        if member.premium_since.is_some() {
            embed = embed.field("💎 Boosting", "Yes", true);
        }
    }

    // Add user avatar
    embed = embed.thumbnail(target_user.face());

    // Add user banner if available (requires additional API call for full user)
    if let Ok(full_user) = ctx.http().get_user(target_user.id).await {
        if let Some(banner_url) = full_user.banner_url() {
            embed = embed.image(banner_url);
        }
    }

    embed = embed.footer(CreateEmbedFooter::new("User Information"));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

pub fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, secs)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
