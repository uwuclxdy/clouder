use anyhow::Result;
use sqlx::{Sqlite, migrate::MigrateDatabase, sqlite::SqliteConnectOptions, sqlite::SqlitePool};
use std::{path::Path, str::FromStr};

use tracing::info;

pub mod dashboard_users;
pub mod guild_cache;
pub mod guild_configs;
pub mod mediaonly;
pub mod reminders;
pub mod selfroles;
pub mod uwufy;
pub mod welcome_goodbye;

pub async fn initialize_database(db_url: &str) -> Result<SqlitePool> {
    let data_dir = "data";
    let db_path = db_url;

    if !Path::new(data_dir).exists() {
        std::fs::create_dir_all(data_dir)?;
    }

    if !Sqlite::database_exists(db_path).await? {
        Sqlite::create_database(db_path).await?;
        info!("created db: {}", db_path);
    }

    let options = SqliteConnectOptions::from_str(db_path)?.pragma("foreign_keys", "ON");
    let pool = SqlitePool::connect_with(options).await?;

    run_migrations(&pool).await?;

    Ok(pool)
}

// sets up the database
async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    let migrations = [
        include_str!("../../migrations/001_initial.sql"),
        include_str!("../../migrations/002_reminders.sql"),
        include_str!("../../migrations/003_welcome_goodbye.sql"),
        include_str!("../../migrations/004_mediaonly.sql"),
        include_str!("../../migrations/005_uwufy.sql"),
        include_str!("../../migrations/006_selfrole_labels.sql"),
        include_str!("../../migrations/007_dashboard_users.sql"),
        include_str!("../../migrations/008_fix_reminder_unique.sql"),
        include_str!("../../migrations/009_custom_reminders.sql"),
    ];

    for migration_content in migrations {
        info!(
            "running migration {}",
            migration_content
                .lines()
                .next()
                .unwrap()
                .trim_start_matches("-- ")
        );
        for statement in migration_content.split(';') {
            let statement: &str = statement.trim();
            if !statement.is_empty() {
                sqlx::query(statement).execute(pool).await?;
            }
        }
    }

    info!("db migrations ok");
    Ok(())
}
