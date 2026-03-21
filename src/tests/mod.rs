pub mod about_tests;
mod channel_tests;
pub mod commands_tests;
pub mod config_tests;
pub mod database_tests;
pub mod events_tests;
mod github_tests;
mod help_tests;
mod huggingface_tests;
mod mediaonly_tests;
mod purge_tests;
mod reminders_tests;
mod shared_tests;
pub mod utils_tests;
mod uwufy_tests;
mod welcome_goodbye_tests;

use clouder_core::config::AppState;
use serenity::all::Http;
use sqlx::SqlitePool;
use std::sync::Arc;

/// Create a test database for testing
pub async fn create_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        r#"
        CREATE TABLE selfrole_configs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            guild_id TEXT NOT NULL,
            channel_id TEXT NOT NULL,
            message_id TEXT UNIQUE,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            selection_type TEXT NOT NULL CHECK(selection_type IN ('radio', 'multiple')),
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE selfrole_roles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            config_id INTEGER NOT NULL,
            role_id TEXT NOT NULL,
            emoji TEXT NOT NULL,
            FOREIGN KEY (config_id) REFERENCES selfrole_configs(id) ON DELETE CASCADE
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE selfrole_cooldowns (
            user_id TEXT NOT NULL,
            role_id TEXT NOT NULL,
            guild_id TEXT NOT NULL,
            expires_at DATETIME NOT NULL,
            PRIMARY KEY (user_id, role_id, guild_id)
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE mediaonly_configs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            guild_id TEXT NOT NULL,
            channel_id TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT 1,
            allow_links BOOLEAN NOT NULL DEFAULT 1,
            allow_attachments BOOLEAN NOT NULL DEFAULT 1,
            allow_gifs BOOLEAN NOT NULL DEFAULT 1,
            allow_stickers BOOLEAN NOT NULL DEFAULT 1,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(guild_id, channel_id)
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE uwufy_toggles (
            guild_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            toggled_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY(guild_id, user_id)
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // reminder tables (mirror migration 002_reminders.sql)
    sqlx::query(
        r#"
        CREATE TABLE user_settings (
            user_id TEXT PRIMARY KEY,
            timezone TEXT NOT NULL DEFAULT 'UTC',
            dm_reminders_enabled BOOLEAN NOT NULL DEFAULT TRUE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE guild_configs (
            guild_id TEXT PRIMARY KEY,
            command_prefix TEXT NOT NULL DEFAULT '!',
            embed_color INTEGER DEFAULT NULL,
            timezone TEXT NOT NULL DEFAULT 'UTC',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE reminder_configs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            guild_id TEXT NOT NULL,
            reminder_type TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT FALSE,
            channel_id TEXT,
            message_type TEXT NOT NULL DEFAULT 'embed',
            message_content TEXT,
            embed_title TEXT,
            embed_description TEXT,
            embed_color INTEGER,
            wysi_morning_time TEXT DEFAULT '07:27',
            wysi_evening_time TEXT DEFAULT '19:27',
            timezone TEXT NOT NULL DEFAULT 'UTC',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(guild_id, reminder_type)
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE reminder_ping_roles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            config_id INTEGER NOT NULL,
            role_id TEXT NOT NULL
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE reminder_subscriptions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT NOT NULL,
            config_id INTEGER NOT NULL,
            subscribed_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE reminder_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            config_id INTEGER NOT NULL,
            execution_time DATETIME NOT NULL,
            status TEXT NOT NULL,
            error_message TEXT,
            channel_sent BOOLEAN DEFAULT FALSE,
            dm_count INTEGER DEFAULT 0,
            dm_failed_count INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE custom_reminders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            guild_id TEXT NOT NULL,
            name TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT FALSE,
            channel_id TEXT,
            schedule_time TEXT NOT NULL DEFAULT '12:00',
            schedule_days TEXT NOT NULL DEFAULT '',
            timezone TEXT NOT NULL DEFAULT 'UTC',
            message_type TEXT NOT NULL DEFAULT 'embed',
            message_content TEXT,
            embed_title TEXT,
            embed_description TEXT,
            embed_color INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE custom_reminder_ping_roles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            reminder_id INTEGER NOT NULL,
            role_id TEXT NOT NULL
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE custom_reminder_subscriptions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT NOT NULL,
            reminder_id INTEGER NOT NULL,
            subscribed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE (user_id, reminder_id)
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE custom_reminder_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            reminder_id INTEGER NOT NULL,
            execution_time DATETIME NOT NULL,
            status TEXT NOT NULL,
            error_message TEXT,
            channel_sent BOOLEAN DEFAULT FALSE,
            dm_count INTEGER DEFAULT 0,
            dm_failed_count INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

/// Create a test AppState
pub async fn create_test_app_state() -> AppState {
    let config = Arc::new(clouder_core::config::Config::test_config());
    let db = Arc::new(create_test_db().await);
    let http = Arc::new(Http::new("test_token"));

    AppState::new(config, db, http)
}
