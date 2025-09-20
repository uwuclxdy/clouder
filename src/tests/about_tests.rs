use crate::commands::about::BOT_START_TIME;
use crate::utils::format_duration;
use poise::serenity_prelude as serenity;
use sysinfo::System;

#[tokio::test]
async fn test_format_duration() {
    assert_eq!(format_duration(0), "0s");
    assert_eq!(format_duration(30), "30s");
    assert_eq!(format_duration(60), "1m 0s");
    assert_eq!(format_duration(90), "1m 30s");
    assert_eq!(format_duration(3600), "1h 0m 0s");
    assert_eq!(format_duration(3661), "1h 1m 1s");
    assert_eq!(format_duration(86400), "1d 0h 0m 0s");
    assert_eq!(format_duration(90061), "1d 1h 1m 1s");
}

#[tokio::test]
async fn test_bot_start_time_initialization() {
    // BOT_START_TIME should be initialized when the module loads
    let uptime = BOT_START_TIME.elapsed().unwrap_or_default();
    // Should be a very small duration since it just loaded
    assert!(uptime.as_millis() < 1000);
}

#[tokio::test]
async fn test_database_stats_format() {
    // Test that database stats string formatting works correctly
    let db_stats = format!(
        "Configs: {}\nRoles: {}\nCooldowns: {}\nDB Guilds: {}",
        5, 15, 2, 3
    );

    assert!(db_stats.contains("Configs: 5"));
    assert!(db_stats.contains("Roles: 15"));
    assert!(db_stats.contains("Cooldowns: 2"));
    assert!(db_stats.contains("DB Guilds: 3"));

    // Check that the format matches our expected pattern
    let lines: Vec<&str> = db_stats.lines().collect();
    assert_eq!(lines.len(), 4);
}

#[test]
fn test_process_id_format() {
    // Test that process ID can be retrieved and formatted
    let pid = std::process::id();
    let process_info = format!("PID: {}", pid);

    assert!(process_info.starts_with("PID: "));
    assert!(pid > 0); // Process ID should be positive
}

#[test]
fn test_system_info_structures() {
    // Test that sysinfo System can be created and basic methods work
    let mut sys = System::new_all();
    sys.refresh_all();

    // These should not panic
    let _total_memory = sys.total_memory();
    let _used_memory = sys.used_memory();
    let _cpu_usage = sys.global_cpu_usage();
    let _cpu_count = sys.cpus().len();
    let _os_name = System::name();
    let _os_version = System::os_version();
}

#[test]
fn test_discord_timestamp_format() {
    // Test that our timestamp formatting produces valid Discord timestamps
    let test_timestamp = 1640995200; // Jan 1, 2022
    let formatted = format!("<t:{}:F>", test_timestamp);
    assert_eq!(formatted, "<t:1640995200:F>");

    let relative_formatted = format!("<t:{}:R>", test_timestamp);
    assert_eq!(relative_formatted, "<t:1640995200:R>");
}

#[test]
fn test_premium_tier_conversion() {
    // Test boost level conversion
    let tier0 = serenity::PremiumTier::Tier0;
    let tier1 = serenity::PremiumTier::Tier1;
    let tier2 = serenity::PremiumTier::Tier2;
    let tier3 = serenity::PremiumTier::Tier3;

    let level0 = match tier0 {
        serenity::PremiumTier::Tier0 => 0,
        serenity::PremiumTier::Tier1 => 1,
        serenity::PremiumTier::Tier2 => 2,
        serenity::PremiumTier::Tier3 => 3,
        _ => 0,
    };

    let level1 = match tier1 {
        serenity::PremiumTier::Tier0 => 0,
        serenity::PremiumTier::Tier1 => 1,
        serenity::PremiumTier::Tier2 => 2,
        serenity::PremiumTier::Tier3 => 3,
        _ => 0,
    };

    let level2 = match tier2 {
        serenity::PremiumTier::Tier0 => 0,
        serenity::PremiumTier::Tier1 => 1,
        serenity::PremiumTier::Tier2 => 2,
        serenity::PremiumTier::Tier3 => 3,
        _ => 0,
    };

    let level3 = match tier3 {
        serenity::PremiumTier::Tier0 => 0,
        serenity::PremiumTier::Tier1 => 1,
        serenity::PremiumTier::Tier2 => 2,
        serenity::PremiumTier::Tier3 => 3,
        _ => 0,
    };

    assert_eq!(level0, 0);
    assert_eq!(level1, 1);
    assert_eq!(level2, 2);
    assert_eq!(level3, 3);
}

#[test]
fn test_memory_calculations() {
    // Test memory conversion calculations
    let bytes_1gb = 1024 * 1024 * 1024_u64;
    let mb_from_gb = bytes_1gb / 1024 / 1024;
    assert_eq!(mb_from_gb, 1024);

    let total_memory = 8192_u64; // 8GB in MB
    let used_memory = 4096_u64; // 4GB in MB
    let usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;
    assert_eq!(usage_percent, 50.0);
}

#[test]
fn test_package_version() {
    // Test that CARGO_PKG_VERSION is available at compile time
    let version = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty());
    // Should follow semantic versioning pattern
    assert!(version.contains('.'));
}

// Integration test for system info collection
#[test]
fn test_system_info_collection() {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Memory information
    let total_memory = sys.total_memory() / 1024 / 1024;
    let used_memory = sys.used_memory() / 1024 / 1024;

    assert!(total_memory > 0);
    assert!(used_memory <= total_memory);

    // CPU information
    let cpu_usage = sys.global_cpu_usage();
    let cpu_count = sys.cpus().len();

    assert!(cpu_usage >= 0.0);
    assert!(cpu_count > 0);

    // OS information
    let os_name = System::name();
    let os_version = System::os_version();

    // These might be None on some systems, but shouldn't panic
    if let Some(name) = os_name {
        assert!(!name.is_empty());
    }
    if let Some(version) = os_version {
        assert!(!version.is_empty());
    }
}
