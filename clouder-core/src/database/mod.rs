use anyhow::Result;
use sqlx::{Sqlite, migrate::MigrateDatabase, sqlite::SqliteConnectOptions, sqlite::SqlitePool};
use std::{path::Path, str::FromStr};

use tracing::info;

pub mod dashboard_sessions;
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
        include_str!("../../migrations/010_dashboard_users_security.sql"),
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
        for statement in split_sql_statements(migration_content) {
            let statement = statement.trim();
            if !statement.is_empty() {
                sqlx::query(statement).execute(pool).await?;
            }
        }
    }

    info!("db migrations ok");
    Ok(())
}

/// Splits a migration file into statements without breaking on `;` characters
/// that live inside `--` line comments or `'…'` string literals. The previous
/// naive `split(';')` mis-parsed any comment containing a semicolon, turning
/// the comment tail into a fake SQL statement.
fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut chars = sql.chars().peekable();
    let mut in_string = false;
    let mut in_line_comment = false;

    while let Some(c) = chars.next() {
        if in_line_comment {
            current.push(c);
            if c == '\n' {
                in_line_comment = false;
            }
            continue;
        }
        if in_string {
            current.push(c);
            if c == '\'' {
                // SQL doubled-quote escape: '' inside a string literal.
                if chars.peek() == Some(&'\'') {
                    current.push(chars.next().unwrap());
                } else {
                    in_string = false;
                }
            }
            continue;
        }
        match c {
            '\'' => {
                in_string = true;
                current.push(c);
            }
            '-' if chars.peek() == Some(&'-') => {
                in_line_comment = true;
                current.push(c);
            }
            ';' => {
                out.push(std::mem::take(&mut current));
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        out.push(current);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::split_sql_statements;

    #[test]
    fn ignores_semicolon_in_line_comment() {
        let sql = "-- a; b\nSELECT 1;SELECT 2;";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 2);
        assert!(stmts[0].contains("SELECT 1"));
        assert_eq!(stmts[1].trim(), "SELECT 2");
    }

    #[test]
    fn ignores_semicolon_in_string_literal() {
        let sql = "INSERT INTO t VALUES ('a;b');SELECT 1;";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 2);
        assert!(stmts[0].contains("'a;b'"));
    }

    #[test]
    fn handles_doubled_quote_escape() {
        let sql = "INSERT INTO t VALUES ('it''s; fine');SELECT 1;";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 2);
    }
}
