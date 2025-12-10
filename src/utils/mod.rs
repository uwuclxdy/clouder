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

/// Generate a Discord user avatar URL with fallback
pub fn get_user_avatar_url(user_id: &str, avatar_hash: Option<&String>) -> String {
    avatar_hash
        .map(|hash| {
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png",
                user_id, hash
            )
        })
        .unwrap_or_else(|| "https://cdn.discordapp.com/embed/avatars/0.png".to_string())
}

/// Generate a Discord guild icon URL with fallback
pub fn get_guild_icon_url(guild_id: &str, icon_hash: Option<&String>) -> String {
    icon_hash
        .map(|hash| format!("https://cdn.discordapp.com/icons/{}/{}.png", guild_id, hash))
        .unwrap_or_else(|| "https://cdn.discordapp.com/embed/avatars/0.png".to_string())
}

/// Generate the Discord bot invite URL
pub fn get_bot_invite_url(client_id: &str, redirect_uri: Option<&str>) -> String {
    match redirect_uri {
        Some(uri) => format!(
            "https://discord.com/oauth2/authorize?client_id={}&permissions=268697088&response_type=code&redirect_uri={}&integration_type=0&scope=bot",
            client_id, uri
        ),
        None => format!(
            "https://discord.com/oauth2/authorize?client_id={}&permissions=268697088&scope=bot%20applications.commands",
            client_id
        ),
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

pub fn can_bot_manage_roles_in_guild(
    bot_member: &serenity::all::Member,
    guild_roles: &[serenity::all::Role],
) -> (bool, Vec<u16>) {
    if bot_member.permissions.unwrap_or_default().administrator() {
        return (true, vec![]);
    }
    let bot_role_positions = get_bot_role_positions(bot_member, guild_roles);

    (false, bot_role_positions)
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

/// Returns discord formatted time
#[allow(dead_code)]
pub fn format_discord_timestamp(time: &str, style: char) -> String {
    let date_time = match chrono::DateTime::parse_from_rfc3339(time) {
        Ok(dt) => dt,
        Err(_) => return "invalid timestamp".to_string(),
    };
    let timestamp = date_time.timestamp();
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

/// Simple channel info for UI display
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChannelInfo {
    pub id: String,
    pub name: String,
}

/// Get text channels from a guild
pub async fn get_guild_text_channels(
    http: &Http,
    guild_id: &str,
) -> Result<Vec<ChannelInfo>, Box<dyn std::error::Error + Send + Sync>> {
    let guild_id_u64: u64 = guild_id.parse()?;
    let channels = http.get_channels(guild_id_u64.into()).await?;

    let text_channels: Vec<ChannelInfo> = channels
        .into_iter()
        .filter(|channel| matches!(channel.kind, serenity::all::ChannelType::Text))
        .map(|channel| ChannelInfo {
            id: channel.id.to_string(),
            name: channel.name,
        })
        .collect();

    Ok(text_channels)
}
