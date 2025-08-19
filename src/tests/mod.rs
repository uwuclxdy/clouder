pub mod config_tests;
pub mod database_tests;
pub mod web_tests;
pub mod events_tests;
pub mod commands_tests;
pub mod utils_tests;

// Test utilities and common setup functions
use crate::config::AppState;
use sqlx::SqlitePool;
use std::sync::Arc;
use serenity::all::{Cache, Http};
use tempfile::NamedTempFile;

/// Create a test database for testing
pub async fn create_test_db() -> SqlitePool {
    // Use in-memory database for tests
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    
    // Run migrations manually instead of using sqlx::migrate!
    sqlx::query(r#"
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
    "#).execute(&pool).await.unwrap();
    
    sqlx::query(r#"
        CREATE TABLE selfrole_roles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            config_id INTEGER NOT NULL,
            role_id TEXT NOT NULL,
            emoji TEXT NOT NULL,
            FOREIGN KEY (config_id) REFERENCES selfrole_configs(id) ON DELETE CASCADE
        );
    "#).execute(&pool).await.unwrap();
    
    sqlx::query(r#"
        CREATE TABLE selfrole_cooldowns (
            user_id TEXT NOT NULL,
            role_id TEXT NOT NULL,
            guild_id TEXT NOT NULL,
            expires_at DATETIME NOT NULL,
            PRIMARY KEY (user_id, role_id, guild_id)
        );
    "#).execute(&pool).await.unwrap();
    
    pool
}

/// Create a test AppState
pub async fn create_test_app_state() -> AppState {
    let config = Arc::new(crate::config::Config::test_config());
    let db = Arc::new(create_test_db().await);
    let cache = Arc::new(Cache::new());
    let http = Arc::new(Http::new("test_token"));
    
    AppState::new(config, db, cache, http)
}