use crate::utils::parse_sqlite_datetime;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaOnlyConfig {
    pub id: i64,
    pub guild_id: String,
    pub channel_id: String,
    pub enabled: bool,
    pub allow_links: bool,
    pub allow_attachments: bool,
    pub allow_gifs: bool,
    pub allow_stickers: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl MediaOnlyConfig {
    pub async fn get_by_channel(
        pool: &SqlitePool,
        guild_id: &str,
        channel_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, guild_id, channel_id, enabled, allow_links, allow_attachments,
                   allow_gifs, allow_stickers, created_at, updated_at
            FROM mediaonly_configs
            WHERE guild_id = ? AND channel_id = ?
            "#,
        )
        .bind(guild_id)
        .bind(channel_id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Self {
                id: row.get("id"),
                guild_id: row.get("guild_id"),
                channel_id: row.get("channel_id"),
                enabled: row.get("enabled"),
                allow_links: row.get("allow_links"),
                allow_attachments: row.get("allow_attachments"),
                allow_gifs: row.get("allow_gifs"),
                allow_stickers: row.get("allow_stickers"),
                created_at: parse_sqlite_datetime(&row.get::<String, _>("created_at")),
                updated_at: parse_sqlite_datetime(&row.get::<String, _>("updated_at")),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_by_guild(pool: &SqlitePool, guild_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, guild_id, channel_id, enabled, allow_links, allow_attachments,
                   allow_gifs, allow_stickers, created_at, updated_at
            FROM mediaonly_configs
            WHERE guild_id = ?
            ORDER BY created_at
            "#,
        )
        .bind(guild_id)
        .fetch_all(pool)
        .await?;

        let mut configs = Vec::new();
        for row in rows {
            configs.push(Self {
                id: row.get("id"),
                guild_id: row.get("guild_id"),
                channel_id: row.get("channel_id"),
                enabled: row.get("enabled"),
                allow_links: row.get("allow_links"),
                allow_attachments: row.get("allow_attachments"),
                allow_gifs: row.get("allow_gifs"),
                allow_stickers: row.get("allow_stickers"),
                created_at: parse_sqlite_datetime(&row.get::<String, _>("created_at")),
                updated_at: parse_sqlite_datetime(&row.get::<String, _>("updated_at")),
            });
        }

        Ok(configs)
    }

    pub async fn upsert(
        pool: &SqlitePool,
        guild_id: &str,
        channel_id: &str,
        enabled: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO mediaonly_configs (guild_id, channel_id, enabled, allow_links, allow_attachments, allow_gifs, allow_stickers)
            VALUES (?, ?, ?, TRUE, TRUE, TRUE, TRUE)
            ON CONFLICT(guild_id, channel_id) DO UPDATE SET
                enabled = excluded.enabled,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(guild_id)
        .bind(channel_id)
        .bind(enabled)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn toggle(
        pool: &SqlitePool,
        guild_id: &str,
        channel_id: &str,
    ) -> Result<bool, sqlx::Error> {
        // First check if config exists
        if let Some(config) = Self::get_by_channel(pool, guild_id, channel_id).await? {
            let new_enabled = !config.enabled;
            Self::upsert(pool, guild_id, channel_id, new_enabled).await?;
            Ok(new_enabled)
        } else {
            // Create new config, enabled by default
            Self::upsert(pool, guild_id, channel_id, true).await?;
            Ok(true)
        }
    }

    pub async fn update_permissions(
        pool: &SqlitePool,
        guild_id: &str,
        channel_id: &str,
        allow_links: bool,
        allow_attachments: bool,
        allow_gifs: bool,
        allow_stickers: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE mediaonly_configs
            SET allow_links = ?, allow_attachments = ?, allow_gifs = ?, allow_stickers = ?, updated_at = CURRENT_TIMESTAMP
            WHERE guild_id = ? AND channel_id = ?
            "#,
        )
        .bind(allow_links)
        .bind(allow_attachments)
        .bind(allow_gifs)
        .bind(allow_stickers)
        .bind(guild_id)
        .bind(channel_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(
        pool: &SqlitePool,
        guild_id: &str,
        channel_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM mediaonly_configs
            WHERE guild_id = ? AND channel_id = ?
            "#,
        )
        .bind(guild_id)
        .bind(channel_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
