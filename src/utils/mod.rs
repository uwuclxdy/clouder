use crate::config::AppState;
use serenity::all::Color;

pub mod welcome_goodbye;

/// Get the default embed color from configuration
pub fn get_default_embed_color(app_state: &AppState) -> Color {
    Color::new(app_state.config.web.embed.default_color)
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
