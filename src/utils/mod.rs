pub mod embed;

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn validate_role_hierarchy(
    bot_highest_role_position: i16,
    target_role_position: i16,
) -> bool {
    bot_highest_role_position > target_role_position
}

/// Check if the bot can manage a target role by checking if ANY of the bot's roles
/// is higher than the target role in the hierarchy
pub fn can_bot_manage_role(
    bot_role_positions: &[u16],
    target_role_position: u16,
) -> bool {
    bot_role_positions.iter()
        .any(|&bot_pos| bot_pos > target_role_position)
}

/// Check if a bot member can manage roles, considering admin permissions
pub fn can_bot_manage_roles_in_guild(
    bot_member: &serenity::all::Member,
    guild_roles: &[serenity::all::Role],
) -> (bool, Vec<u16>) {
    // Check if bot has administrator permission
    if bot_member.permissions.unwrap_or_default().administrator() {
        return (true, vec![]); // Admin can manage all roles, return empty positions list
    }

    // Get all bot role positions for hierarchy checking
    let bot_role_positions = get_bot_role_positions(bot_member, guild_roles);

    (false, bot_role_positions)
}

/// Get all role positions for a bot member based on their roles in the guild
pub fn get_bot_role_positions(
    bot_member: &serenity::all::Member,
    guild_roles: &[serenity::all::Role],
) -> Vec<u16> {
    bot_member.roles.iter()
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
        'F' => format!("<t:{}:F>", timestamp), // Tuesday, August 19, 2025 at 04:05:00 PM
        'f' => format!("<t:{}:f>", timestamp), // August 19, 2025 at 04:05 PM
        'D' => format!("<t:{}:D>", timestamp), // Tuesday, August 19, 2025
        'd' => format!("<t:{}:d>", timestamp), // 08/19/2025
        't' => format!("<t:{}:t>", timestamp), // 04:05 PM
        'T' => format!("<t:{}:T>", timestamp), // 04:05:00 PM
        'R' => format!("<t:{}:R>", timestamp), // relative time
        _ => format!("<t:{}:f>", timestamp),   // default to brief format
    }
}
