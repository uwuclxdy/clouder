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

    let start = std::time::Instant::now();
    let _ = ctx.http().get_current_user().await;
    let api_latency = start.elapsed();

    let start = std::time::Instant::now();
    let _ = ctx.ping().await;
    let gateway_latency = start.elapsed();

    let mut sys = System::new_all();
    sys.refresh_all();

    let total_memory = sys.total_memory() / 1024 / 1024;
    let used_memory = sys.used_memory() / 1024 / 1024;
    let available_memory = sys.available_memory() / 1024 / 1024;
    let memory_percentage = (used_memory as f64 / total_memory as f64) * 100.0;

    let cpu_count = sys.cpus().len();
    let cpu_usage = sys.global_cpu_usage();

    let current_pid = std::process::id();

    let disk_info = "Storage: Available";

    let guild_count = ctx.cache().guild_count();
    let cached_users = ctx.cache().user_count();

    let mut cached_channels = 0;
    for guild_id in ctx.cache().guilds() {
        if let Some(guild) = ctx.cache().guild(guild_id) {
            cached_channels += guild.channels.len();
        }
    }

    let bot_user = ctx.http().get_current_user().await?;

    let db_stats = get_enhanced_database_stats(ctx).await;

    let os_info = format!("{} {}",
        System::name().unwrap_or_else(|| "Unknown".to_string()),
        System::os_version().unwrap_or_else(|| "Unknown".to_string())
    );
    let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());

    let build_info = format!(
        "**Framework:** Serenity + Poise\n**Language:** Rust\n**Target:** {}",
        std::env::consts::ARCH
    );

    let memory_per_guild = if guild_count > 0 {
        format!("{:.1} MB per guild", used_memory as f64 / guild_count as f64)
    } else {
        "N/A".to_string()
    };

    let embed = CreateEmbed::new()
        .title("🤖 clouder bot - system info")
        .description(format!(
            "**{}** • id: `{}` • v**{}**",
            bot_user.tag(),
            bot_user.id,
            bot_version
        ))
        .color(Color::BLITZ_BLUE)
        .thumbnail(bot_user.face())

        .field(
            "⚡ performance stuff",
            format!(
                "🕐 **uptime:** {}\n⏱️ **api latency:** {}ms\n🌐 **gateway latency:** {}ms",
                uptime_str,
                api_latency.as_millis(),
                gateway_latency.as_millis()
            ),
            true
        )

        .field(
            "📊 discord stats",
            format!(
                "🏰 **guilds:** {}\n👥 **cached users:** {}\n💬 **cached channels:** {}",
                guild_count,
                cached_users,
                cached_channels
            ),
            true
        )

        .field(
            "🗄️ database stats",
            db_stats,
            true
        )

        .field(
            "💾 memory usage",
            format!(
                "**used:** {:.1}% ({} mb)\n**available:** {} mb\n**total:** {} mb\n**per guild:** {}",
                memory_percentage,
                used_memory,
                available_memory,
                total_memory,
                memory_per_guild
            ),
            true
        )

        .field(
            "⚙️ cpu info",
            format!(
                "**usage:** {:.1}%\n**cores:** {}\n**arch:** {}",
                cpu_usage,
                cpu_count,
                std::env::consts::ARCH
            ),
            true
        )

        .field(
            "💿 storage status",
            format!(
                "{}",
                disk_info
            ),
            true
        )

        .field(
            "🖥️ system env",
            format!(
                "**os:** {}\n**kernel:** {}\n**hostname:** {}\n**pid:** {}",
                os_info,
                kernel_version,
                hostname,
                current_pid
            ),
            false
        )

        .field(
            "🔧 runtime info",
            format!(
                "**process id:** {}\n**os:** {}\n**family:** {}",
                current_pid,
                std::env::consts::OS,
                std::env::consts::FAMILY
            ),
            true
        )

        .field(
            "📦 build info",
            build_info,
            true
        )

        .field(
            "👨‍💻 developer",
            "**made by:** [uwuclxdy](https://github.com/uwuclxdy)\n**repo:** [clouder](https://github.com/uwuclxdy/clouder)",
            true
        )

        .footer(CreateEmbedFooter::new("clouder bot • all systems operational 🟢"))
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

async fn get_enhanced_database_stats(ctx: Context<'_>) -> String {
    let db = &ctx.data().db;

    let selfrole_configs = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM selfrole_configs")
        .fetch_one(db.as_ref())
        .await.unwrap_or_else(|_| 0);

    let selfrole_roles = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM selfrole_roles")
        .fetch_one(db.as_ref())
        .await.unwrap_or_else(|_| 0);

    let active_cooldowns = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM selfrole_cooldowns WHERE expires_at > datetime('now')")
        .fetch_one(db.as_ref())
        .await.unwrap_or_else(|_| 0);

    let db_guilds = sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT guild_id) FROM selfrole_configs")
        .fetch_one(db.as_ref())
        .await.unwrap_or_else(|_| 0);

    let recent_configs = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM selfrole_configs WHERE created_at > datetime('now', '-7 days')"
    )
    .fetch_one(db.as_ref())
    .await.unwrap_or_else(|_| 0);

    let expired_cooldowns = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM selfrole_cooldowns WHERE expires_at <= datetime('now')")
        .fetch_one(db.as_ref())
        .await.unwrap_or_else(|_| 0);

    format!(
        "**configs:** {}\n**roles:** {}\n**active cooldowns:** {}\n**db guilds:** {}\n**recent (7d):** {}\n**expired:** {}",
        selfrole_configs, selfrole_roles, active_cooldowns, db_guilds, recent_configs, expired_cooldowns
    )
}

#[poise::command(slash_command)]
pub async fn server(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.send(poise::CreateReply::default()
                .content("❌ this command can only be used in a server!")
                .ephemeral(true))
                .await?;
            return Ok(());
        }
    };

    let full_guild = match ctx.http().get_guild(guild_id).await {
        Ok(guild) => guild,
        Err(_) => {
            ctx.send(poise::CreateReply::default()
                .content("❌ failed to fetch server info!")
                .ephemeral(true))
                .await?;
            return Ok(());
        }
    };

    let member_count = match full_guild.approximate_member_count {
        Some(count) => count,
        None => {
            match ctx.guild() {
                Some(guild) => guild.member_count,
                None => 0,
            }
        }
    };

    let created_at = guild_id.created_at();
    let created_timestamp = format!("<t:{}:F>", created_at.timestamp());

    let channels = if let Ok(channels) = full_guild.channels(&ctx.http()).await {
        channels
    } else {
        std::collections::HashMap::new()
    };

    let text_channels = channels.values().filter(|c| matches!(c.kind, serenity::ChannelType::Text)).count();
    let voice_channels = channels.values().filter(|c| matches!(c.kind, serenity::ChannelType::Voice)).count();
    let total_channels = channels.len();

    let role_count = full_guild.roles.len();

    let boost_level = match full_guild.premium_tier {
        serenity::PremiumTier::Tier0 => 0,
        serenity::PremiumTier::Tier1 => 1,
        serenity::PremiumTier::Tier2 => 2,
        serenity::PremiumTier::Tier3 => 3,
        _ => 0,
    };
    let boost_count = full_guild.premium_subscription_count.unwrap_or(0);

    let owner = match full_guild.owner_id.to_user(&ctx.http()).await {
        Ok(user) => format!("{} ({})", user.tag(), user.id),
        Err(_) => format!("Unknown ({})", full_guild.owner_id),
    };

    let features = if full_guild.features.is_empty() {
        "None".to_string()
    } else {
        full_guild.features.iter()
            .map(|f| format!("{:?}", f))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let mut embed = CreateEmbed::new()
        .title(&format!("📊 {} server info", full_guild.name))
        .color(Color::PURPLE)
        .field("👥 members", member_count.to_string(), true)
        .field("💬 channels", format!("{} ({} text, {} voice)", total_channels, text_channels, voice_channels), true)
        .field("🎭 roles", role_count.to_string(), true)
        .field("👑 owner", owner, false)
        .field("📅 created", created_timestamp, true)
        .field("🚀 boost level", format!("level {} ({} boosts)", boost_level, boost_count), true)
        .field("🏷️ server id", guild_id.to_string(), true);

    if features != "None" {
        embed = embed.field("✨ features", features, false);
    }

    if let Some(icon_url) = full_guild.icon_url() {
        embed = embed.thumbnail(icon_url);
    }

    if let Some(banner_url) = full_guild.banner_url() {
        embed = embed.image(banner_url);
    }

    embed = embed.footer(CreateEmbedFooter::new("server stats"));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn user(
    ctx: Context<'_>,
    #[description = "User to get information about"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user = user.unwrap_or_else(|| ctx.author().clone());

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
        .title(&format!("👤 {} user info", target_user.tag()))
        .color(Color::BLUE)
        .field("🏷️ user id", target_user.id.to_string(), true)
        .field("📅 account created", account_age, false);

    if target_user.bot {
        embed = embed.field("🤖 bot", "yes", true);
    }

    if let Some(member) = member_info {
        if let Some(joined_at) = member.joined_at {
            let join_info = format!("<t:{}:F> (<t:{}:R>)", joined_at.timestamp(), joined_at.timestamp());
            embed = embed.field("📥 joined server", join_info, false);
        }

        let display_name = member.display_name().to_string();
        if display_name != target_user.name {
            embed = embed.field("📝 nickname", display_name, true);
        }

        // exclude @everyone
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

        if member.premium_since.is_some() {
            embed = embed.field("💎 boosting", "yes", true);
        }
    }

    embed = embed.thumbnail(target_user.face());

    if let Ok(full_user) = ctx.http().get_user(target_user.id).await {
        if let Some(banner_url) = full_user.banner_url() {
            embed = embed.image(banner_url);
        }
    }

    embed = embed.footer(CreateEmbedFooter::new("user info"));

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
