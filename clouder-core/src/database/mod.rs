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

async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    let migrations = [
        Migration::new(1, include_str!("../../migrations/001_initial.sql")),
        Migration::new(2, include_str!("../../migrations/002_reminders.sql")),
        Migration::new(3, include_str!("../../migrations/003_welcome_goodbye.sql")),
        Migration::new(4, include_str!("../../migrations/004_mediaonly.sql")),
        Migration::new(5, include_str!("../../migrations/005_uwufy.sql")),
        Migration::new(6, include_str!("../../migrations/006_selfrole_labels.sql")),
        Migration::new(7, include_str!("../../migrations/007_dashboard_users.sql")),
        Migration::new(
            8,
            include_str!("../../migrations/008_fix_reminder_unique.sql"),
        ),
        Migration::new(9, include_str!("../../migrations/009_custom_reminders.sql")),
        Migration::new(
            10,
            include_str!("../../migrations/010_dashboard_users_security.sql"),
        ),
        Migration::new(11, include_str!("../../migrations/011_guild_cache_ttl.sql")),
        Migration::new(
            12,
            include_str!("../../migrations/012_drop_legacy_api_key.sql"),
        ),
        Migration::new(
            13,
            include_str!("../../migrations/013_dashboard_users_api_key_ciphertext.sql"),
        ),
    ];

    create_migration_ledger(pool).await?;
    recover_partial_migrations(pool).await?;
    bootstrap_migration_ledger(pool).await?;

    for migration in migrations {
        if migration_applied(pool, migration.version).await? {
            continue;
        }

        info!("running migration {}", migration.name);
        let mut tx = pool.begin().await?;
        for statement in split_sql_statements(migration.content) {
            let statement = statement.trim();
            if !statement.is_empty() {
                sqlx::query(statement).execute(&mut *tx).await?;
            }
        }
        sqlx::query("INSERT OR IGNORE INTO schema_migrations (version, name) VALUES (?, ?)")
            .bind(migration.version)
            .bind(migration.name)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
    }

    info!("db migrations ok");
    Ok(())
}

struct Migration {
    version: i64,
    name: &'static str,
    content: &'static str,
}

impl Migration {
    fn new(version: i64, content: &'static str) -> Self {
        Self {
            version,
            name: migration_name(content),
            content,
        }
    }
}

fn migration_name(content: &'static str) -> &'static str {
    content
        .lines()
        .next()
        .unwrap_or("migration")
        .trim_start_matches("-- ")
}

async fn create_migration_ledger(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version    INTEGER PRIMARY KEY NOT NULL,
            name       TEXT NOT NULL,
            applied_at INTEGER NOT NULL DEFAULT (unixepoch())
        )",
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn recover_partial_migrations(pool: &SqlitePool) -> Result<()> {
    let new_exists = table_exists(pool, "dashboard_users_new").await?;
    let old_exists = table_exists(pool, "dashboard_users").await?;

    if new_exists && !old_exists {
        sqlx::query("ALTER TABLE dashboard_users_new RENAME TO dashboard_users")
            .execute(pool)
            .await?;
        sqlx::query(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_dashboard_users_api_key_hash
                ON dashboard_users(api_key_hash) WHERE api_key_hash IS NOT NULL",
        )
        .execute(pool)
        .await?;
        record_migration(pool, 12, "recovered partial migration").await?;
    } else if new_exists {
        sqlx::query("DROP TABLE dashboard_users_new")
            .execute(pool)
            .await?;
    }

    Ok(())
}

async fn bootstrap_migration_ledger(pool: &SqlitePool) -> Result<()> {
    if migration_applied(pool, 1).await? {
        return Ok(());
    }

    if table_exists(pool, "dashboard_users").await? {
        for version in 1..=9 {
            record_migration(pool, version, "pre-ledger migration").await?;
        }
    }

    if dashboard_security_migration_started(pool).await? {
        ensure_dashboard_security_migration(pool).await?;
        record_migration(pool, 10, "pre-ledger migration").await?;
    }

    if column_exists(pool, "user_guild_cache", "expires_at").await? {
        record_migration(pool, 11, "pre-ledger migration").await?;
    }

    if table_exists(pool, "dashboard_users").await?
        && !column_exists(pool, "dashboard_users", "api_key").await?
    {
        record_migration(pool, 12, "pre-ledger migration").await?;
    }

    if column_exists(pool, "dashboard_users", "api_key_ciphertext").await? {
        record_migration(pool, 13, "pre-ledger migration").await?;
    }

    Ok(())
}

async fn dashboard_security_migration_started(pool: &SqlitePool) -> Result<bool> {
    Ok(
        column_exists(pool, "dashboard_users", "api_key_hash").await?
            || table_exists(pool, "dashboard_sessions").await?,
    )
}

async fn ensure_dashboard_security_migration(pool: &SqlitePool) -> Result<()> {
    if !column_exists(pool, "dashboard_users", "api_key_hash").await? {
        sqlx::query("ALTER TABLE dashboard_users ADD COLUMN api_key_hash TEXT")
            .execute(pool)
            .await?;
    }

    for column in [
        "oauth_token TEXT",
        "oauth_token_updated_at INTEGER",
        "username TEXT",
        "avatar TEXT",
    ] {
        let Some((column_name, _)) = column.split_once(' ') else {
            continue;
        };
        if !column_exists(pool, "dashboard_users", column_name).await? {
            sqlx::query(&format!("ALTER TABLE dashboard_users ADD COLUMN {column}"))
                .execute(pool)
                .await?;
        }
    }

    sqlx::query("DROP INDEX IF EXISTS idx_dashboard_users_api_key")
        .execute(pool)
        .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_dashboard_users_api_key_hash
            ON dashboard_users(api_key_hash) WHERE api_key_hash IS NOT NULL",
    )
    .execute(pool)
    .await?;
    if column_exists(pool, "dashboard_users", "api_key").await? {
        sqlx::query("UPDATE dashboard_users SET api_key = user_id")
            .execute(pool)
            .await?;
    }
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS dashboard_sessions (
            session_id TEXT    PRIMARY KEY NOT NULL,
            user_id    TEXT    NOT NULL,
            csrf_token TEXT    NOT NULL,
            expires_at INTEGER NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (unixepoch())
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_dashboard_sessions_user_id
            ON dashboard_sessions(user_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_dashboard_sessions_expires_at
            ON dashboard_sessions(expires_at)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn migration_applied(pool: &SqlitePool, version: i64) -> Result<bool> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schema_migrations WHERE version = ?")
        .bind(version)
        .fetch_one(pool)
        .await?;

    Ok(count > 0)
}

async fn record_migration(pool: &SqlitePool, version: i64, name: &str) -> Result<()> {
    sqlx::query("INSERT OR IGNORE INTO schema_migrations (version, name) VALUES (?, ?)")
        .bind(version)
        .bind(name)
        .execute(pool)
        .await?;

    Ok(())
}

async fn table_exists(pool: &SqlitePool, table_name: &str) -> Result<bool> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?")
            .bind(table_name)
            .fetch_one(pool)
            .await?;

    Ok(count > 0)
}

async fn column_exists(pool: &SqlitePool, table_name: &str, column_name: &str) -> Result<bool> {
    let escaped_table_name = table_name.replace('"', "\"\"");
    let columns: Vec<String> = sqlx::query_scalar(&format!(
        "SELECT name FROM pragma_table_info(\"{escaped_table_name}\")"
    ))
    .fetch_all(pool)
    .await?;

    Ok(columns.iter().any(|column| column == column_name))
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
    use super::{run_migrations, split_sql_statements};
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn runs_migrations_once() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        run_migrations(&pool).await.unwrap();
        run_migrations(&pool).await.unwrap();

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schema_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 13);
    }

    #[tokio::test]
    async fn recovers_dashboard_users_rename_after_partial_run() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();

        sqlx::query("INSERT INTO dashboard_users (user_id) VALUES ('u1')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("ALTER TABLE dashboard_users RENAME TO dashboard_users_new")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM schema_migrations WHERE version = 12")
            .execute(&pool)
            .await
            .unwrap();

        run_migrations(&pool).await.unwrap();

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM dashboard_users WHERE user_id = 'u1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 1);
        let stale: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'dashboard_users_new'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(stale, 0);
    }

    #[tokio::test]
    async fn repairs_partially_applied_dashboard_security_migration() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        for migration in [
            include_str!("../../migrations/001_initial.sql"),
            include_str!("../../migrations/002_reminders.sql"),
            include_str!("../../migrations/003_welcome_goodbye.sql"),
            include_str!("../../migrations/004_mediaonly.sql"),
            include_str!("../../migrations/005_uwufy.sql"),
            include_str!("../../migrations/006_selfrole_labels.sql"),
            include_str!("../../migrations/007_dashboard_users.sql"),
            include_str!("../../migrations/008_fix_reminder_unique.sql"),
            include_str!("../../migrations/009_custom_reminders.sql"),
        ] {
            for statement in split_sql_statements(migration) {
                let statement = statement.trim();
                if !statement.is_empty() {
                    sqlx::query(statement).execute(&pool).await.unwrap();
                }
            }
        }
        sqlx::query("ALTER TABLE dashboard_users ADD COLUMN api_key_hash TEXT")
            .execute(&pool)
            .await
            .unwrap();

        run_migrations(&pool).await.unwrap();

        for column in [
            "api_key_hash",
            "oauth_token",
            "oauth_token_updated_at",
            "username",
            "avatar",
        ] {
            let count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM pragma_table_info('dashboard_users') WHERE name = ?",
            )
            .bind(column)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(count, 1);
        }
    }

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
