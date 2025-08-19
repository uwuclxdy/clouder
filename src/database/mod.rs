use sqlx::{sqlite::SqlitePool, Sqlite, migrate::MigrateDatabase};
use std::path::Path;
use anyhow::Result;

pub mod selfroles;

pub async fn initialize_database(db_url: &str) -> Result<SqlitePool> {
    let data_dir = "data";
    let db_path = db_url;
    
    if !Path::new(data_dir).exists() {
        std::fs::create_dir_all(data_dir)?;
    }
    
    if !Sqlite::database_exists(db_path).await? {
        Sqlite::create_database(db_path).await?;
        tracing::info!("Created new database at {}", db_path);
    }
    
    let pool = SqlitePool::connect(db_path).await?;
    
    create_tables(&pool).await?;
    
    Ok(pool)
}

async fn create_tables(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS selfrole_configs (
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
        "#
    ).execute(pool).await?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS selfrole_roles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            config_id INTEGER NOT NULL,
            role_id TEXT NOT NULL,
            emoji TEXT NOT NULL,
            FOREIGN KEY (config_id) REFERENCES selfrole_configs(id) ON DELETE CASCADE
        );
        "#
    ).execute(pool).await?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS selfrole_cooldowns (
            user_id TEXT NOT NULL,
            role_id TEXT NOT NULL,
            guild_id TEXT NOT NULL,
            expires_at DATETIME NOT NULL,
            PRIMARY KEY (user_id, role_id, guild_id)
        );
        "#
    ).execute(pool).await?;
    
    tracing::info!("Database tables created/verified");
    Ok(())
}