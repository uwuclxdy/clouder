use crate::config::AppState;
use serenity::all::{Channel, ChannelId, Color, GuildId, Http, Permissions};

pub mod content_detection;
pub mod welcome_goodbye;

/// Get the default embed color from configuration
pub fn get_default_embed_color(app_state: &AppState) -> Color {
    Color::new(app_state.config.web.embed.default_color)
}

/// Result of checking bot permissions in a channel
pub struct BotChannelPermissions {
    pub permissions: Permissions,
}

/// Check bot permissions in a specific channel
/// Returns None if any lookup fails, otherwise returns the permissions
pub async fn get_bot_channel_permissions(
    http: &Http,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Option<BotChannelPermissions> {
    let bot_user = http.get_current_user().await.ok()?;
    let bot_member = http.get_member(guild_id, bot_user.id).await.ok()?;
    let guild = http.get_guild(guild_id).await.ok()?;
    let channel = http.get_channel(channel_id).await.ok()?;

    let guild_channel = match channel {
        Channel::Guild(gc) => gc,
        _ => return None,
    };

    let permissions = guild.user_permissions_in(&guild_channel, &bot_member);

    Some(BotChannelPermissions { permissions })
}

/// Check if bot has a specific permission in a channel (for DMs, returns true)
pub async fn bot_has_permission_in_channel(
    http: &Http,
    guild_id: Option<GuildId>,
    channel_id: ChannelId,
    permission_check: impl Fn(&Permissions) -> bool,
) -> bool {
    match guild_id {
        Some(gid) => {
            if let Some(perms) = get_bot_channel_permissions(http, gid, channel_id).await {
                permission_check(&perms.permissions)
            } else {
                false
            }
        }
        None => true, // DMs - assume all permissions
    }
}

/// Parse datetime string from SQLite format, with fallback to current time
pub fn parse_sqlite_datetime(datetime_str: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now())
}

pub fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

pub fn can_bot_manage_role(bot_role_positions: &[u16], target_role_position: u16) -> bool {
    bot_role_positions
        .iter()
        .any(|&bot_pos| bot_pos > target_role_position)
}

pub fn get_bot_role_positions(
    bot_member: &serenity::all::Member,
    guild_roles: &[serenity::all::Role],
) -> Vec<u16> {
    bot_member
        .roles
        .iter()
        .filter_map(|role_id| guild_roles.iter().find(|r| r.id == *role_id))
        .map(|role| role.position)
        .collect()
}

pub fn discord_timestamp(timestamp: i64, style: char) -> String {
    match style {
        'F' => format!("<t:{}:F>", timestamp),
        'f' => format!("<t:{}:f>", timestamp),
        'D' => format!("<t:{}:D>", timestamp),
        'd' => format!("<t:{}:d>", timestamp),
        't' => format!("<t:{}:t>", timestamp),
        'T' => format!("<t:{}:T>", timestamp),
        'R' => format!("<t:{}:R>", timestamp),
        _ => format!("<t:{}:f>", timestamp),
    }
}
