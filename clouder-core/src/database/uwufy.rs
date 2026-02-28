use crate::utils::parse_sqlite_datetime;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UwufyToggle {
    pub guild_id: String,
    pub user_id: String,
    pub enabled: bool,
    pub toggled_at: chrono::DateTime<chrono::Utc>,
}

impl UwufyToggle {
    pub async fn get(
        pool: &SqlitePool,
        guild_id: &str,
        user_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT guild_id, user_id, enabled, toggled_at
            FROM uwufy_toggles
            WHERE guild_id = ? AND user_id = ?
            "#,
        )
        .bind(guild_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|row| Self {
            guild_id: row.get("guild_id"),
            user_id: row.get("user_id"),
            enabled: row.get("enabled"),
            toggled_at: parse_sqlite_datetime(&row.get::<String, _>("toggled_at")),
        }))
    }

    pub async fn is_enabled(
        pool: &SqlitePool,
        guild_id: &str,
        user_id: &str,
    ) -> Result<bool, sqlx::Error> {
        Ok(Self::get(pool, guild_id, user_id)
            .await?
            .map(|t| t.enabled)
            .unwrap_or(false))
    }

    pub async fn toggle(
        pool: &SqlitePool,
        guild_id: &str,
        user_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let new_enabled = if let Some(existing) = Self::get(pool, guild_id, user_id).await? {
            !existing.enabled
        } else {
            true
        };

        sqlx::query(
            r#"
            INSERT INTO uwufy_toggles (guild_id, user_id, enabled)
            VALUES (?, ?, ?)
            ON CONFLICT(guild_id, user_id) DO UPDATE SET
                enabled = excluded.enabled,
                toggled_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(guild_id)
        .bind(user_id)
        .bind(new_enabled)
        .execute(pool)
        .await?;

        Ok(new_enabled)
    }

    pub async fn get_enabled_in_guild(
        pool: &SqlitePool,
        guild_id: &str,
    ) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT user_id
            FROM uwufy_toggles
            WHERE guild_id = ? AND enabled = TRUE
            "#,
        )
        .bind(guild_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.get("user_id")).collect())
    }

    pub async fn disable_all_in_guild(
        pool: &SqlitePool,
        guild_id: &str,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE uwufy_toggles
            SET enabled = FALSE, toggled_at = CURRENT_TIMESTAMP
            WHERE guild_id = ? AND enabled = TRUE
            "#,
        )
        .bind(guild_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn set_enabled(
        pool: &SqlitePool,
        guild_id: &str,
        user_id: &str,
        enabled: bool,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO uwufy_toggles (guild_id, user_id, enabled)
            VALUES (?, ?, ?)
            ON CONFLICT(guild_id, user_id) DO UPDATE SET
                enabled = excluded.enabled,
                toggled_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(guild_id)
        .bind(user_id)
        .bind(enabled)
        .execute(pool)
        .await?;

        Ok(enabled)
    }
}
