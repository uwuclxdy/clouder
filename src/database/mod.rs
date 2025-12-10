use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Sqlite};
use std::path::Path;

use crate::logging::info;

pub mod mediaonly;
pub mod selfroles;
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

    let pool = SqlitePool::connect(db_path).await?;

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
    ];

    for (index, migration_content) in migrations.iter().enumerate() {
        info!(
            "running migration {}",
            migrations[index]
                .split('/')
                .next_back()
                .unwrap_or("unknown")
        );
        for statement in migration_content.split(';') {
            let statement = statement.trim();
            if !statement.is_empty() {
                sqlx::query(statement).execute(pool).await?;
            }
        }
    }

    info!("db migrations ok");
    Ok(())
}
