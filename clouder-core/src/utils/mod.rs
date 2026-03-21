use crate::config::AppState;
use serenity::all::Color;

pub mod content_detection;
pub mod welcome_goodbye;

/// Resolves the embed color for an optional guild.
/// Priority: guild DB override → global env default → hardcoded fallback.
pub async fn get_embed_color(app_state: &AppState, guild_id: Option<u64>) -> Color {
    use crate::database::guild_configs::GuildConfig;

    if let Some(gid) = guild_id
        && let Ok(config) = GuildConfig::get_or_default(&app_state.db, &gid.to_string()).await
        && let Some(hex) = config.embed_color
    {
        let stripped = hex.trim_start_matches('#');
        if let Ok(n) = u32::from_str_radix(stripped, 16) {
            return Color::new(n);
        }
    }
    Color::new(app_state.config.web.embed.default_color)
}

/// Parse datetime string from SQLite format, with fallback to current time
pub fn parse_sqlite_datetime(datetime_str: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc())
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

/// check whether `perms` satisfies `required`, taking administrator into account.
///
/// serenity's `Permissions` bitflags do not consider `ADMINISTRATOR` to imply all
/// other permissions; we need to explicitly treat `administrator` as overriding the
/// requirement. this helper exists so that permission checks across the codebase
/// remain consistent and easier to test.
///
/// returning `true` means the caller has the required permission or is an admin.
pub fn has_permission(
    perms: serenity::all::Permissions,
    required: serenity::all::Permissions,
) -> bool {
    // administrator bypasses everything
    if perms.administrator() {
        return true;
    }
    perms.contains(required)
}

pub fn parse_hhmm(s: &str) -> Option<chrono::NaiveTime> {
    let (hours, minutes) = s.split_once(':')?;

    if hours.is_empty() || minutes.is_empty() || minutes.contains(':') {
        return None;
    }

    let hours = hours.parse::<u32>().ok()?;
    let minutes = minutes.parse::<u32>().ok()?;
    chrono::NaiveTime::from_hms_opt(hours, minutes, 0)
}

pub fn is_valid_hhmm(s: &str) -> bool {
    parse_hhmm(s).is_some()
}

pub fn is_valid_url(s: &str) -> bool {
    s.starts_with("https://") || s.starts_with("http://")
}

pub fn format_count(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
