use crate::config::AppState;
use crate::utils::{format_duration, get_default_embed_color};
use anyhow::Result;
use lazy_static::lazy_static;
use poise::serenity_prelude as serenity;
use serenity::CreateEmbed;
use std::time::SystemTime;
use sysinfo::System;

lazy_static! {
    pub static ref BOT_START_TIME: SystemTime = SystemTime::now();
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

#[poise::command(slash_command, subcommands("bot", "server", "user", "role", "channel"))]
pub async fn about(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn bot(ctx: Context<'_>) -> Result<(), Error> {
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

    let guild_count = ctx.cache().guild_count();
    let cached_users = ctx.cache().user_count();

    let mut cached_channels = 0;
    for guild_id in ctx.cache().guilds() {
        if let Some(guild) = ctx.cache().guild(guild_id) {
            cached_channels += guild.channels.len();
        }
    }

    let bot_user = ctx.http().get_current_user().await?;

    let db = &ctx.data().db;

    let selfrole_configs = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM selfrole_configs")
        .fetch_one(db.as_ref())
        .await
        .unwrap_or(0);

    let selfrole_roles = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM selfrole_roles")
        .fetch_one(db.as_ref())
        .await
        .unwrap_or(0);

    let active_cooldowns = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM selfrole_cooldowns WHERE expires_at > datetime('now')",
    )
    .fetch_one(db.as_ref())
    .await
    .unwrap_or(0);

    let db_guilds =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT guild_id) FROM selfrole_configs")
            .fetch_one(db.as_ref())
            .await
            .unwrap_or(0);

    let recent_configs = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM selfrole_configs WHERE created_at > datetime('now', '-7 days')",
    )
    .fetch_one(db.as_ref())
    .await
    .unwrap_or(0);

    let expired_cooldowns = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM selfrole_cooldowns WHERE expires_at <= datetime('now')",
    )
    .fetch_one(db.as_ref())
    .await
    .unwrap_or(0);

    let db_stats = format!(
        "configs: **`{}`**
        roles: **`{}`**
        active cooldowns: **`{}`**
        servers: **`{}`**
        recent (7d): **`{}`**
        expired: **`{}`**",
        selfrole_configs,
        selfrole_roles,
        active_cooldowns,
        db_guilds,
        recent_configs,
        expired_cooldowns
    );

    let os_info = format!(
        "{} {}",
        System::name().unwrap_or_else(|| "Unknown".to_string()),
        System::os_version().unwrap_or_else(|| "Unknown".to_string())
    );
    let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());

    let embed = CreateEmbed::new()
        .title("info")
        .description(format!(
            "<@{}> `{}`
            v{}",
            bot_user.id, bot_user.id, bot_version
        ))
        .color(get_default_embed_color(ctx.data()))
        .thumbnail(bot_user.face())
        .field(
            "performance",
            format!(
                "uptime: **`{}`**
                api: **`{}ms`**
                gateway: **`{}ms`**",
                uptime_str,
                api_latency.as_millis(),
                gateway_latency.as_millis()
            ),
            true,
        )
        .field(
            "discord stats",
            format!(
                "servers: **`{}`**
                **cached:**
                users: **`{}`**
                channels: **`{}`**",
                guild_count, cached_users, cached_channels
            ),
            true,
        )
        .field("database", db_stats, true)
        .field(
            "CPU",
            format!(
                "usage: **{:.1}%**
                cores: **`{}`**
                arch: **`{}`**",
                cpu_usage,
                cpu_count,
                std::env::consts::ARCH
            ),
            true,
        )
        .field(
            "memory",
            format!(
                "used: **{:.1}% ({}MB)**
                free: **{:.3}MB**
                total: **{:.3}MB**",
                memory_percentage, used_memory, available_memory, total_memory
            ),
            true,
        )
        .field(
            "system",
            format!(
                "**`{}`**
                **`{}`**
                bot pid: **`{}`**",
                os_info, kernel_version, current_pid
            ),
            true,
        )
        .field(
            "build",
            format!(
                "**Rust** (Serenity + Poise)
                arch: **`{}`**",
                std::env::consts::ARCH
            ),
            true,
        )
        .field(
            "vibecoder",
            "**[uwuclxdy](https://github.com/uwuclxdy)**
            bot is FOSS btw: **[clouder](https://github.com/uwuclxdy/clouder)**
            **Claude 4 Sonnet <3**",
            true,
        )
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn server(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.send(
                poise::CreateReply::default()
                    .content("this command can only be used in a server!")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let full_guild = match ctx.http().get_guild_with_counts(guild_id).await {
        Ok(guild) => guild,
        Err(_) => match ctx.http().get_guild(guild_id).await {
            Ok(guild) => guild,
            Err(_) => {
                ctx.send(
                    poise::CreateReply::default()
                        .content("could not fetch server info")
                        .ephemeral(true),
                )
                .await?;
                return Ok(());
            }
        },
    };

    let member_count = match full_guild.approximate_member_count {
        Some(count) => count,
        None => match ctx.guild() {
            Some(guild) => guild.member_count,
            None => 0,
        },
    };

    let created_at = guild_id.created_at();
    let created_timestamp = format!("<t:{}:F>", created_at.timestamp());

    let channels = full_guild.channels(&ctx.http()).await.unwrap_or_default();

    let text_channels = channels
        .values()
        .filter(|c| matches!(c.kind, serenity::ChannelType::Text))
        .count();
    let voice_channels = channels
        .values()
        .filter(|c| matches!(c.kind, serenity::ChannelType::Voice))
        .count();
    let stage_channels = channels
        .values()
        .filter(|c| matches!(c.kind, serenity::ChannelType::Stage))
        .count();
    let forum_channels = channels
        .values()
        .filter(|c| matches!(c.kind, serenity::ChannelType::Forum))
        .count();
    let category_channels = channels
        .values()
        .filter(|c| matches!(c.kind, serenity::ChannelType::Category))
        .count();
    let total_channels = channels.len();

    let role_count = full_guild.roles.len();
    let emoji_count = full_guild.emojis.len();
    let sticker_count = full_guild.stickers.len();

    let boost_level = match full_guild.premium_tier {
        serenity::PremiumTier::Tier0 => 0,
        serenity::PremiumTier::Tier1 => 1,
        serenity::PremiumTier::Tier2 => 2,
        serenity::PremiumTier::Tier3 => 3,
        _ => 0,
    };
    let boost_count = full_guild.premium_subscription_count.unwrap_or(0);

    let owner = match full_guild.owner_id.to_user(&ctx.http()).await {
        Ok(user) => format!("<@{}> `{}`", user.id, user.id),
        Err(_) => format!("unknown (`{}`)", full_guild.owner_id),
    };

    let verification_level = match full_guild.verification_level {
        serenity::VerificationLevel::None => "None",
        serenity::VerificationLevel::Low => "Low",
        serenity::VerificationLevel::Medium => "Medium",
        serenity::VerificationLevel::High => "High",
        serenity::VerificationLevel::Higher => "Highest",
        _ => "Unknown",
    };

    let explicit_filter = format!("{:?}", full_guild.explicit_content_filter);

    let notification_level = format!("{:?}", full_guild.default_message_notifications);

    let mut features = Vec::new();
    for feature in &full_guild.features {
        let feature_str = match feature.as_str() {
            "COMMUNITY" => "Community",
            "PARTNERED" => "Partnered",
            "VERIFIED" => "Verified",
            "DISCOVERABLE" => "Discoverable",
            "VANITY_URL" => "Vanity URL",
            "BANNER" => "Banner",
            "ANIMATED_BANNER" => "Animated Banner",
            "INVITE_SPLASH" => "Invite Splash",
            "VIP_REGIONS" => "VIP Voice Regions",
            "WELCOME_SCREEN_ENABLED" => "Welcome Screen",
            "THREADS_ENABLED" => "Threads",
            "PRIVATE_THREADS" => "Private Threads",
            "ROLE_ICONS" => "Role Icons",
            "NEWS" => "News Channels",
            "PUBLIC" => "Public",
            "MONETIZATION_ENABLED" => "Monetized",
            _ => continue,
        };
        features.push(feature_str);
    }

    let mut embed = CreateEmbed::new()
        .title(format!("`{}` info", full_guild.name))
        .color(get_default_embed_color(ctx.data()))
        .field("members", format!("**`{member_count}`**"), true);

    if let Some(approximate_presence_count) = full_guild.approximate_presence_count {
        embed = embed.field(
            "online",
            format!("**~`{}`**", approximate_presence_count),
            true,
        );
    }

    if let Some(description) = &full_guild.description
        && !description.is_empty()
    {
        embed = embed.field("description", format!("*{}*", description), false);
    }

    let channel_breakdown = if stage_channels > 0 || forum_channels > 0 {
        format!(
            "**`{total_channels}`** total\n**`{text_channels}`** text\n**`{voice_channels}`** voice\n**`{stage_channels}`** stage\n**`{forum_channels}`** forum\n**`{category_channels}`** categories"
        )
    } else {
        format!(
            "**`{total_channels}`** total\n**`{text_channels}`** text\n**`{voice_channels}`** voice\n**`{category_channels}`** categories"
        )
    };

    embed = embed.field("channels", channel_breakdown, true).field(
        "roles",
        format!("**`{role_count}`**"),
        true,
    );

    if emoji_count > 0 || sticker_count > 0 {
        embed = embed.field(
            "emojis & stickers",
            format!("emojis: **`{emoji_count}`**\nstickers: **`{sticker_count}`**"),
            true,
        );
    }

    embed = embed
        .field("owner", owner, false)
        .field("created", created_timestamp, false)
        .field(
            "boost level",
            format!("boosts: **`{boost_count}`**\nlevel: **`{boost_level}`**"),
            true,
        );

    if let Some(vanity_url) = &full_guild.vanity_url_code {
        embed = embed.field("vanity url", format!("**discord.gg/{}**", vanity_url), true);
    }

    embed = embed.field(
        "security",
        format!(
            "verification: **{}**\nfilter: **{}**\nnotifications: **{}**",
            verification_level, explicit_filter, notification_level
        ),
        true,
    );

    if !features.is_empty() && features.len() <= 6 {
        embed = embed.field("features", features.join(" "), false);
    }

    if let Some(max_members) = full_guild.max_members {
        embed = embed.field("max members", format!("**`{}`**", max_members), true);
    }

    if let Some(max_presences) = full_guild.max_presences {
        embed = embed.field("max presences", format!("**`{}`**", max_presences), true);
    }

    if let Some(max_video_channel_users) = full_guild.max_video_channel_users {
        embed = embed.field(
            "max video users",
            format!("**`{}`**", max_video_channel_users),
            true,
        );
    }

    if let Some(max_stage_video_channel_users) = full_guild.max_stage_video_channel_users {
        embed = embed.field(
            "max stage users",
            format!("**`{}`**", max_stage_video_channel_users),
            true,
        );
    }

    if let Some(widget_enabled) = full_guild.widget_enabled
        && widget_enabled
    {
        embed = embed.field("widget enabled", "**yes**", true);
        if let Some(widget_channel_id) = full_guild.widget_channel_id {
            embed = embed.field("widget channel", format!("<#{}>", widget_channel_id), true);
        }
    }

    if let Some(application_id) = full_guild.application_id {
        embed = embed.field("created by app", format!("`{}`", application_id), true);
    }

    if let Some(system_channel_id) = full_guild.system_channel_id {
        embed = embed.field("system channel", format!("<#{}>", system_channel_id), true);

        let system_flags = full_guild.system_channel_flags;
        let mut suppressed = Vec::new();
        if system_flags.contains(serenity::SystemChannelFlags::SUPPRESS_JOIN_NOTIFICATIONS) {
            suppressed.push("join messages");
        }
        if system_flags.contains(serenity::SystemChannelFlags::SUPPRESS_PREMIUM_SUBSCRIPTIONS) {
            suppressed.push("boost messages");
        }
        if system_flags
            .contains(serenity::SystemChannelFlags::SUPPRESS_GUILD_REMINDER_NOTIFICATIONS)
        {
            suppressed.push("setup tips");
        }
        if system_flags.contains(serenity::SystemChannelFlags::SUPPRESS_JOIN_NOTIFICATION_REPLIES) {
            suppressed.push("join replies");
        }
        if system_flags.contains(
            serenity::SystemChannelFlags::SUPPRESS_ROLE_SUBSCRIPTION_PURCHASE_NOTIFICATIONS,
        ) {
            suppressed.push("role subscription purchases");
        }
        if system_flags.contains(
            serenity::SystemChannelFlags::SUPPRESS_ROLE_SUBSCRIPTION_PURCHASE_NOTIFICATION_REPLIES,
        ) {
            suppressed.push("role subscription replies");
        }

        if !suppressed.is_empty() {
            embed = embed.field("suppressed", suppressed.join(", "), true);
        }
    }

    if let Some(rules_channel_id) = full_guild.rules_channel_id {
        embed = embed.field("rules channel", format!("<#{}>", rules_channel_id), true);
    }

    if let Some(public_updates_channel_id) = full_guild.public_updates_channel_id {
        embed = embed.field(
            "updates channel",
            format!("<#{}>", public_updates_channel_id),
            true,
        );
    }

    let mfa_level = match full_guild.mfa_level {
        serenity::MfaLevel::None => "None",
        serenity::MfaLevel::Elevated => "Elevated (2FA required)",
        _ => "Unknown",
    };
    embed = embed.field("2FA requirement", format!("**{}**", mfa_level), true);

    let nsfw_level = match full_guild.nsfw_level {
        serenity::NsfwLevel::Default => "Default",
        serenity::NsfwLevel::Explicit => "Explicit",
        serenity::NsfwLevel::Safe => "Safe",
        serenity::NsfwLevel::AgeRestricted => "Age Restricted",
        _ => "Unknown",
    };
    embed = embed.field("NSFW level", format!("**{}**", nsfw_level), true);

    embed = embed.field(
        "locale",
        format!("**`{}`**", full_guild.preferred_locale),
        true,
    );

    if full_guild.premium_progress_bar_enabled {
        embed = embed.field("progress bar", "**enabled**", true);
    }

    embed = embed.field("server id", format!("**`{guild_id}`**"), true);

    if let Some(afk_metadata) = &full_guild.afk_metadata {
        embed = embed.field(
            "AFK channel",
            format!("<#{}>", afk_metadata.afk_channel_id),
            true,
        );
        let timeout_secs = match afk_metadata.afk_timeout {
            serenity::AfkTimeout::OneMinute => 60,
            serenity::AfkTimeout::FiveMinutes => 300,
            serenity::AfkTimeout::FifteenMinutes => 900,
            serenity::AfkTimeout::ThirtyMinutes => 1800,
            serenity::AfkTimeout::OneHour => 3600,
            _ => 0,
        };
        if timeout_secs > 0 {
            embed = embed.field("AFK timeout", format!("**{}min**", timeout_secs / 60), true);
        }
    }

    if let Some(welcome_screen) = &full_guild.welcome_screen
        && let Some(description) = &welcome_screen.description
        && !description.is_empty()
    {
        embed = embed.field("welcome description", format!("*{}*", description), false);
    }

    if let Some(discovery_splash) = &full_guild.discovery_splash {
        let discovery_splash_url = format!(
            "https://cdn.discordapp.com/discovery-splashes/{}/{}.png",
            guild_id, discovery_splash
        );
        embed = embed.field(
            "discovery splash",
            format!("[View Image]({})", discovery_splash_url),
            true,
        );
    }

    if let Some(splash) = &full_guild.splash {
        let splash_url = format!(
            "https://cdn.discordapp.com/splashes/{}/{}.png",
            guild_id, splash
        );
        embed = embed.field(
            "invite splash",
            format!("[View Image]({})", splash_url),
            true,
        );
    }

    if let Some(icon_hash) = &full_guild.icon_hash {
        embed = embed.field("icon hash", format!("`{}`", icon_hash), true);
    }

    if let Some(icon_url) = full_guild.icon_url() {
        embed = embed.thumbnail(icon_url);
    }

    if let Some(banner_url) = full_guild.banner_url() {
        embed = embed.image(banner_url);
    }

    embed = embed.timestamp(serenity::Timestamp::now());

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
        (guild_id.member(&ctx.http(), target_user.id).await).ok()
    } else {
        None
    };

    let full_user = ctx
        .http()
        .get_user(target_user.id)
        .await
        .unwrap_or(target_user.clone());

    let created_at = target_user.id.created_at();
    let account_age = format!(
        "<t:{}:F> (<t:{}:R>)",
        created_at.timestamp(),
        created_at.timestamp()
    );

    let display_name = if let Some(global_name) = &full_user.global_name {
        format!("**`{}`** (global name)", global_name)
    } else {
        format!("**`{}`**", target_user.display_name())
    };

    let mut embed = CreateEmbed::new()
        .color(get_default_embed_color(ctx.data()))
        .title(format!("`{}` info", target_user.tag()))
        .description(format!("<@{}> `{}`", target_user.id, target_user.id))
        .field("display name", display_name, true)
        .field("account created", account_age, false);

    if let Some(discriminator) = target_user.discriminator {
        embed = embed.field(
            "discriminator",
            format!("**`#{:04}`**", discriminator),
            true,
        );
    }

    if target_user.bot {
        embed = embed.field("bot", "yes", true);
    }

    if target_user.system {
        embed = embed.field("system user", "yes", true);
    }

    if let Some(locale) = &full_user.locale {
        embed = embed.field("locale", format!("**`{}`**", locale), true);
    }

    if let Some(verified) = full_user.verified {
        embed = embed.field(
            "email verified",
            if verified { "**yes**" } else { "**no**" },
            true,
        );
    }

    if let Some(email) = &full_user.email
        && !email.is_empty()
    {
        embed = embed.field("email", format!("**`{}`**", email), true);
    }

    if full_user.mfa_enabled {
        embed = embed.field("2FA enabled", "**yes**", true);
    }

    let flags = full_user.flags;
    if !flags.is_empty() {
        let mut private_flags = Vec::new();

        if flags.contains(serenity::UserPublicFlags::DISCORD_EMPLOYEE) {
            private_flags.push("Staff");
        }
        if flags.contains(serenity::UserPublicFlags::PARTNERED_SERVER_OWNER) {
            private_flags.push("Partner");
        }
        if flags.contains(serenity::UserPublicFlags::HYPESQUAD_EVENTS) {
            private_flags.push("HypeSquad");
        }
        if flags.contains(serenity::UserPublicFlags::BUG_HUNTER_LEVEL_1) {
            private_flags.push("Bug Hunter L1");
        }
        if flags.contains(serenity::UserPublicFlags::BUG_HUNTER_LEVEL_2) {
            private_flags.push("Bug Hunter L2");
        }
        if flags.contains(serenity::UserPublicFlags::EARLY_VERIFIED_BOT_DEVELOPER) {
            private_flags.push("Verified Dev");
        }
        if flags.contains(serenity::UserPublicFlags::DISCORD_CERTIFIED_MODERATOR) {
            private_flags.push("Cert. Mod");
        }

        if !private_flags.is_empty() {
            embed = embed.field("user flags", private_flags.join(" "), false);
        }
    }

    let user_flags = target_user.public_flags.unwrap_or_default();
    let mut badges = Vec::new();

    if user_flags.contains(serenity::UserPublicFlags::DISCORD_EMPLOYEE) {
        badges.push("Staff");
    }
    if user_flags.contains(serenity::UserPublicFlags::PARTNERED_SERVER_OWNER) {
        badges.push("Partner");
    }
    if user_flags.contains(serenity::UserPublicFlags::HYPESQUAD_EVENTS) {
        badges.push("HypeSquad Events");
    }
    if user_flags.contains(serenity::UserPublicFlags::BUG_HUNTER_LEVEL_1) {
        badges.push("Bug Hunter");
    }
    if user_flags.contains(serenity::UserPublicFlags::BUG_HUNTER_LEVEL_2) {
        badges.push("Bug Hunter Gold");
    }
    if user_flags.contains(serenity::UserPublicFlags::HOUSE_BRAVERY) {
        badges.push("HypeSquad Bravery");
    }
    if user_flags.contains(serenity::UserPublicFlags::HOUSE_BRILLIANCE) {
        badges.push("HypeSquad Brilliance");
    }
    if user_flags.contains(serenity::UserPublicFlags::HOUSE_BALANCE) {
        badges.push("HypeSquad Balance");
    }
    if user_flags.contains(serenity::UserPublicFlags::EARLY_SUPPORTER) {
        badges.push("Early Supporter");
    }
    if user_flags.contains(serenity::UserPublicFlags::EARLY_VERIFIED_BOT_DEVELOPER) {
        badges.push("Verified Bot Developer");
    }
    if user_flags.contains(serenity::UserPublicFlags::DISCORD_CERTIFIED_MODERATOR) {
        badges.push("Certified Moderator");
    }
    if user_flags.contains(serenity::UserPublicFlags::ACTIVE_DEVELOPER) {
        badges.push("Active Developer");
    }

    if !badges.is_empty() {
        embed = embed.field("badges", badges.join(" "), false);
    }

    if full_user.premium_type != serenity::PremiumType::None {
        let nitro_type = match full_user.premium_type {
            serenity::PremiumType::Nitro => "Discord Nitro",
            serenity::PremiumType::NitroClassic => "Discord Nitro Classic",
            _ => "Premium",
        };
        embed = embed.field("subscription", nitro_type, true);
    }

    if let Some(accent_color) = full_user.accent_colour {
        let color_hex = format!("#{:06X}", accent_color.0);
        embed = embed.field("accent color", format!("**`{}`**", color_hex), true);
    }

    if let Some(member) = member_info {
        if let Some(joined_at) = member.joined_at {
            let join_info = format!(
                "<t:{}:F> (<t:{}:R>)",
                joined_at.timestamp(),
                joined_at.timestamp()
            );
            embed = embed.field("joined server", join_info, false);
        }

        let roles: Vec<String> = member
            .roles
            .iter()
            .filter_map(|role_id| {
                if let Some(guild) = ctx.guild() {
                    guild
                        .roles
                        .get(role_id)
                        .map(|role| format!("<@&{}>", role.id))
                } else {
                    None
                }
            })
            .collect();

        if !roles.is_empty() {
            let roles_text = roles.join(" ");
            embed = embed.field(format!("roles: `{}`", roles.len()), roles_text, false);
        }

        if let Some(premium_since) = member.premium_since {
            embed = embed.field(
                "boosting",
                format!("since <t:{}:R>", premium_since.timestamp()),
                true,
            );
        }

        if let Some(timed_out_until) = member.communication_disabled_until
            && timed_out_until > serenity::Timestamp::now()
        {
            embed = embed.field(
                "timed out",
                format!("until <t:{}:R>", timed_out_until.timestamp()),
                true,
            );
        }

        if let Some(nick) = &member.nick {
            embed = embed.field("nickname", format!("**`{}`**", nick), true);
        }

        if member.pending {
            embed = embed.field("membership pending", "yes", true);
        }

        if member.deaf {
            embed = embed.field("server deafened", "yes", true);
        }

        if member.mute {
            embed = embed.field("server muted", "yes", true);
        }

        if let Some(avatar) = &member.avatar {
            let server_avatar_url = format!(
                "https://cdn.discordapp.com/guilds/{}/users/{}/avatars/{}.png?size=1024",
                ctx.guild_id().unwrap(),
                target_user.id,
                avatar
            );
            embed = embed.field(
                "server avatar",
                format!("[View Avatar]({})", server_avatar_url),
                true,
            );
        }
    }

    embed = embed.field("user id", format!("`{}`", target_user.id), true);

    embed = embed.thumbnail(target_user.face());

    if let Some(banner_url) = full_user.banner_url() {
        embed = embed.image(banner_url);
    }

    embed = embed.timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn role(
    ctx: Context<'_>,
    #[description = "Role to get information about"] role: serenity::Role,
) -> Result<(), Error> {
    if ctx.guild_id().is_none() {
        ctx.send(
            poise::CreateReply::default()
                .content("this command can only be used in a server!")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let created_at = role.id.created_at();
    let created_timestamp = format!(
        "<t:{}:F> (<t:{}:R>)",
        created_at.timestamp(),
        created_at.timestamp()
    );

    let member_count = if let Some(guild) = ctx.guild() {
        guild
            .members
            .values()
            .filter(|member| member.roles.contains(&role.id))
            .count()
    } else {
        0
    };

    let permissions_list = if role.permissions.is_empty() {
        "None".to_string()
    } else {
        let mut perms = Vec::new();

        if role.permissions.administrator() {
            perms.push("Administrator");
        } else {
            if role.permissions.manage_guild() {
                perms.push("Manage Server");
            }
            if role.permissions.manage_roles() {
                perms.push("Manage Roles");
            }
            if role.permissions.manage_channels() {
                perms.push("Manage Channels");
            }
            if role.permissions.kick_members() {
                perms.push("Kick Members");
            }
            if role.permissions.ban_members() {
                perms.push("Ban Members");
            }
            if role.permissions.moderate_members() {
                perms.push("Timeout Members");
            }
            if role.permissions.manage_messages() {
                perms.push("Manage Messages");
            }
            if role.permissions.mention_everyone() {
                perms.push("Mention Everyone");
            }
            if role.permissions.view_audit_log() {
                perms.push("View Audit Log");
            }
            if role.permissions.manage_webhooks() {
                perms.push("Manage Webhooks");
            }
            if role.permissions.manage_guild_expressions() {
                perms.push("Manage Emojis");
            }
            if role.permissions.create_instant_invite() {
                perms.push("Create Invites");
            }
            if role.permissions.manage_events() {
                perms.push("Manage Events");
            }
        }

        if perms.len() > 6 {
            format!("{} and {} more", perms[..6].join(", "), perms.len() - 6)
        } else if perms.is_empty() {
            "Basic permissions only".to_string()
        } else {
            perms.join(", ")
        }
    };

    let role_type = if role.managed {
        let tags = &role.tags;
        if tags.bot_id.is_some() {
            "Bot Role"
        } else if tags.integration_id.is_some() {
            "Integration Role"
        } else if tags.premium_subscriber {
            "Booster Role"
        } else if tags.subscription_listing_id.is_some() {
            "Subscription Role"
        } else if tags.available_for_purchase {
            "Purchasable Role"
        } else if tags.guild_connections {
            "Linked Role"
        } else {
            "Managed Role"
        }
    } else {
        "Regular Role"
    };

    let mut embed = CreateEmbed::new()
        .title(format!("`{}` info", role.name))
        .description(format!("<@&{}> `{}`", role.id, role.id))
        .color(role.colour)
        .field("members", format!("**`{}`**", member_count), true)
        .field("position", format!("**`{}`**", role.position), true)
        .field("type", role_type, true)
        .field("created", created_timestamp, false);

    if role.hoist {
        embed = embed.field("displayed separately", "**yes**", true);
    } else {
        embed = embed.field("displayed separately", "**no**", true);
    }

    if role.mentionable {
        embed = embed.field("mentionable", "**yes**", true);
    }

    if role.colour != serenity::Colour::default() {
        let color_hex = format!("#{:06X}", role.colour.0);
        embed = embed.field("color", format!("**`{}`**", color_hex), true);
    }

    if let Some(icon) = &role.icon {
        let icon_url = format!(
            "https://cdn.discordapp.com/role-icons/{}/{}.png",
            role.id, icon
        );
        embed = embed.thumbnail(icon_url);
    }

    if let Some(emoji) = &role.unicode_emoji {
        embed = embed.field("emoji", emoji, true);
    }

    let tags = &role.tags;
    if let Some(bot_id) = tags.bot_id {
        embed = embed.field("bot", format!("<@{}>", bot_id), true);
    }

    if let Some(integration_id) = tags.integration_id {
        embed = embed.field("integration id", format!("`{}`", integration_id), true);
    }

    if let Some(subscription_id) = tags.subscription_listing_id {
        embed = embed.field("subscription id", format!("`{}`", subscription_id), true);
    }

    if tags.available_for_purchase {
        embed = embed.field("available for purchase", "**yes**", true);
    }

    if tags.guild_connections {
        embed = embed.field("linked role", "**yes**", true);
    }

    embed = embed.field("key permissions", permissions_list, false);

    embed = embed.field("role id", format!("`{}`", role.id), true);

    embed = embed.field("guild id", format!("`{}`", role.guild_id), true);

    embed = embed.timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn channel(
    ctx: Context<'_>,
    #[description = "Channel to get information about"] channel: Option<serenity::GuildChannel>,
) -> Result<(), Error> {
    if ctx.guild_id().is_none() {
        ctx.send(
            poise::CreateReply::default()
                .content("❌ this command can only be used in a server!")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let target_channel = match channel {
        Some(ch) => ch,
        None => match ctx.channel_id().to_channel(&ctx.http()).await {
            Ok(serenity::Channel::Guild(guild_channel)) => guild_channel,
            _ => {
                ctx.send(
                    poise::CreateReply::default()
                        .content("could not get channel information!")
                        .ephemeral(true),
                )
                .await?;
                return Ok(());
            }
        },
    };

    let created_at = target_channel.id.created_at();
    let created_timestamp = format!(
        "<t:{}:F> (<t:{}:R>)",
        created_at.timestamp(),
        created_at.timestamp()
    );

    let channel_type = match target_channel.kind {
        serenity::ChannelType::Text => "text channel",
        serenity::ChannelType::Voice => "voice channel",
        serenity::ChannelType::Category => "category",
        serenity::ChannelType::News => "announcement channel",
        serenity::ChannelType::Stage => "stage channel",
        serenity::ChannelType::Forum => "forum channel",
        serenity::ChannelType::PublicThread => "public thread",
        serenity::ChannelType::PrivateThread => "private thread",
        serenity::ChannelType::NewsThread => "news thread",
        _ => "unknown channel type",
    };

    let mut embed = CreateEmbed::new()
        .title(format!("`{}` info", target_channel.name))
        .description(format!("<#{}> `{}`", target_channel.id, target_channel.id))
        .color(get_default_embed_color(ctx.data()))
        .field("type", channel_type, true)
        .field(
            "position",
            format!("**`{}`**", target_channel.position),
            true,
        )
        .field("created", created_timestamp, false);

    embed = embed.field("guild id", format!("`{}`", target_channel.guild_id), true);

    if let Some(topic) = &target_channel.topic
        && !topic.is_empty()
    {
        let display_topic = if topic.len() > 100 {
            format!("{}...", &topic[..97])
        } else {
            topic.clone()
        };
        embed = embed.field("description", format!("*{}*", display_topic), false);
    }

    if let Some(parent_id) = target_channel.parent_id {
        if let Ok(parent) = parent_id.to_channel(&ctx.http()).await {
            if let serenity::Channel::Guild(parent_channel) = parent {
                embed = embed.field(
                    "category",
                    format!("`{}`\n**{}**", parent_channel.id, parent_channel.name),
                    true,
                );
            }
        } else {
            embed = embed.field("parent channel", format!("<#{}>", parent_id), true);
        }
    }

    if target_channel.nsfw {
        embed = embed.field("NSFW", "**yes**", true);
    }

    match target_channel.kind {
        serenity::ChannelType::Text | serenity::ChannelType::News => {
            if let Some(slowmode) = target_channel.rate_limit_per_user
                && slowmode > 0
            {
                embed = embed.field("slowmode", format!("**{}s**", slowmode), true);
            }

            if let Some(last_message_id) = target_channel.last_message_id {
                embed = embed.field(
                    "last message",
                    format!("<t:{}:R>", last_message_id.created_at().timestamp()),
                    true,
                );
            }

            if let Some(last_pin_timestamp) = target_channel.last_pin_timestamp {
                embed = embed.field(
                    "last pin",
                    format!("<t:{}:R>", last_pin_timestamp.timestamp()),
                    true,
                );
            }

            if let Some(default_auto_archive_duration) =
                target_channel.default_auto_archive_duration
            {
                let duration = match default_auto_archive_duration {
                    serenity::AutoArchiveDuration::OneHour => "1 hour",
                    serenity::AutoArchiveDuration::OneDay => "24 hours",
                    serenity::AutoArchiveDuration::ThreeDays => "3 days",
                    serenity::AutoArchiveDuration::OneWeek => "1 week",
                    _ => "custom",
                };
                embed = embed.field("auto archive", format!("**{}**", duration), true);
            }
        }
        serenity::ChannelType::Voice | serenity::ChannelType::Stage => {
            if let Some(bitrate) = target_channel.bitrate {
                embed = embed.field("bitrate", format!("**{}kbps**", bitrate / 1000), true);
            }

            if let Some(user_limit) = target_channel.user_limit {
                if user_limit > 0 {
                    embed = embed.field("user limit", format!("**{}**", user_limit), true);
                } else {
                    embed = embed.field("user limit", "**unlimited**", true);
                }
            }

            if let Some(rtc_region) = &target_channel.rtc_region {
                embed = embed.field("region", format!("**{}**", rtc_region), true);
            }

            if let Some(video_quality_mode) = target_channel.video_quality_mode {
                let quality = match video_quality_mode {
                    serenity::VideoQualityMode::Auto => "Auto",
                    serenity::VideoQualityMode::Full => "720p",
                    _ => "Unknown",
                };
                embed = embed.field("video quality", format!("**{}**", quality), true);
            }
        }
        serenity::ChannelType::Forum => {
            if let Some(slowmode) = target_channel.rate_limit_per_user
                && slowmode > 0
            {
                embed = embed.field("slowmode", format!("**{}s**", slowmode), true);
            }

            let tags = &target_channel.available_tags;
            if !tags.is_empty() {
                let tag_count = tags.len();
                let tag_names: Vec<String> = tags
                    .iter()
                    .take(5)
                    .map(|tag| {
                        if let Some(emoji) = &tag.emoji {
                            match emoji {
                                serenity::ForumEmoji::Name(unicode) => {
                                    format!("{} {}", unicode, tag.name)
                                }
                                serenity::ForumEmoji::Id(id) => {
                                    format!("<:tag:{}> {}", id, tag.name)
                                }
                                _ => tag.name.clone(),
                            }
                        } else {
                            tag.name.clone()
                        }
                    })
                    .collect();

                let display_tags = if tags.len() > 5 {
                    format!("{} (+{} more)", tag_names.join(", "), tags.len() - 5)
                } else {
                    tag_names.join(", ")
                };

                embed = embed.field(format!("tags ({})", tag_count), display_tags, false);
            }

            if let Some(default_reaction_emoji) = &target_channel.default_reaction_emoji {
                let emoji_display = match default_reaction_emoji {
                    serenity::ForumEmoji::Name(unicode) => unicode.clone(),
                    serenity::ForumEmoji::Id(id) => format!("<:reaction:{}>", id),
                    _ => "Unknown".to_string(),
                };
                embed = embed.field("default reaction", emoji_display, true);
            }

            if let Some(default_thread_rate_limit) =
                target_channel.default_thread_rate_limit_per_user
                && default_thread_rate_limit > 0
            {
                embed = embed.field(
                    "thread slowmode",
                    format!("**{}s**", default_thread_rate_limit),
                    true,
                );
            }

            if let Some(default_sort_order) = target_channel.default_sort_order {
                let sort_name = match default_sort_order {
                    serenity::SortOrder::LatestActivity => "Latest Activity",
                    serenity::SortOrder::CreationDate => "Creation Date",
                    _ => "Unknown",
                };
                embed = embed.field("default sort", format!("**{}**", sort_name), true);
            }

            if let Some(default_forum_layout) = target_channel.default_forum_layout {
                let layout_name = match default_forum_layout {
                    serenity::ForumLayoutType::NotSet => "Not Set",
                    serenity::ForumLayoutType::ListView => "List View",
                    serenity::ForumLayoutType::GalleryView => "Gallery View",
                    _ => "Unknown",
                };
                embed = embed.field("layout", format!("**{}**", layout_name), true);
            }
        }
        serenity::ChannelType::PublicThread
        | serenity::ChannelType::PrivateThread
        | serenity::ChannelType::NewsThread => {
            if let Some(thread_metadata) = &target_channel.thread_metadata {
                if thread_metadata.archived {
                    embed = embed.field("archived", "**yes**", true);

                    if let Some(archive_timestamp) = thread_metadata.archive_timestamp {
                        embed = embed.field(
                            "archived at",
                            format!("<t:{}:R>", archive_timestamp.timestamp()),
                            true,
                        );
                    }
                }

                if thread_metadata.locked {
                    embed = embed.field("locked", "**yes**", true);
                }

                let duration = match thread_metadata.auto_archive_duration {
                    serenity::AutoArchiveDuration::OneHour => "1 hour",
                    serenity::AutoArchiveDuration::OneDay => "24 hours",
                    serenity::AutoArchiveDuration::ThreeDays => "3 days",
                    serenity::AutoArchiveDuration::OneWeek => "1 week",
                    _ => "custom",
                };
                embed = embed.field("auto archive", format!("**{}**", duration), true);

                if thread_metadata.invitable {
                    embed = embed.field("invitable", "**yes**", true);
                }
            }

            if let Some(owner_id) = target_channel.owner_id {
                embed = embed.field("thread creator", format!("<@{}>", owner_id), true);
            }

            if let Some(message_count) = target_channel.message_count {
                embed = embed.field("message count", format!("**~{}**", message_count), true);
            }

            if let Some(member_count) = target_channel.member_count {
                embed = embed.field("member count", format!("**~{}**", member_count), true);
            }

            if let Some(total_message_sent) = target_channel.total_message_sent {
                embed = embed.field(
                    "total messages",
                    format!("**{}**", total_message_sent),
                    true,
                );
            }
        }
        _ => {}
    }

    let permission_overwrites = target_channel.permission_overwrites.len();
    if permission_overwrites > 0 {
        let mut overwrite_details = Vec::new();
        for overwrite in &target_channel.permission_overwrites {
            match overwrite.kind {
                serenity::PermissionOverwriteType::Role(role_id) => {
                    overwrite_details.push(format!("Role <@&{}>", role_id));
                }
                serenity::PermissionOverwriteType::Member(user_id) => {
                    overwrite_details.push(format!("User <@{}>", user_id));
                }
                _ => {}
            }
        }

        let overwrite_text = if overwrite_details.len() > 3 {
            format!(
                "{} (+{} more)",
                overwrite_details[..3].join(", "),
                overwrite_details.len() - 3
            )
        } else {
            overwrite_details.join(", ")
        };

        if !overwrite_text.is_empty() {
            embed = embed.field(
                "permission overwrites",
                format!("**`{}`**\n{}", permission_overwrites, overwrite_text),
                false,
            );
        } else {
            embed = embed.field(
                "permission overwrites",
                format!("**`{}`**", permission_overwrites),
                true,
            );
        }
    }

    let flags = target_channel.flags;
    let mut flag_list = Vec::new();
    if flags.contains(serenity::ChannelFlags::PINNED) {
        flag_list.push("Pinned");
    }
    if flags.contains(serenity::ChannelFlags::REQUIRE_TAG) {
        flag_list.push("Requires Tag");
    }

    if !flag_list.is_empty() {
        embed = embed.field("flags", flag_list.join(", "), true);
    }

    embed = embed.timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
