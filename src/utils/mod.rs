// Utility functions for the bot

pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

pub fn validate_role_hierarchy(
    bot_highest_role_position: i16,
    target_role_position: i16,
) -> bool {
    bot_highest_role_position > target_role_position
}