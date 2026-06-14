use anyhow::Result;
use sqlx::SqlitePool;

pub const DEFAULT_TIMEZONE: &str = "UTC";
pub const DEFAULT_COMMAND_PREFIX: &str = "!";

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct GuildConfig {
    pub guild_id: String,
    pub timezone: String,
    pub command_prefix: String,
    pub embed_color: Option<String>,
}

impl GuildConfig {
    pub async fn get_or_default(db: &SqlitePool, guild_id: &str) -> Result<Self> {
        let config = sqlx::query_as::<_, Self>(
            "SELECT guild_id, timezone, command_prefix, embed_color FROM guild_configs WHERE guild_id = ?",
        )
        .bind(guild_id)
        .fetch_optional(db)
        .await?;

        Ok(config.unwrap_or(Self {
            guild_id: guild_id.to_string(),
            timezone: DEFAULT_TIMEZONE.to_string(),
            command_prefix: DEFAULT_COMMAND_PREFIX.to_string(),
            embed_color: None,
        }))
    }

    pub async fn upsert(
        db: &SqlitePool,
        guild_id: &str,
        timezone: &str,
        command_prefix: &str,
        embed_color: Option<&str>,
    ) -> Result<Self> {
        sqlx::query(
            "INSERT INTO guild_configs (guild_id, timezone, command_prefix, embed_color, updated_at)
             VALUES (?, ?, ?, ?, unixepoch())
             ON CONFLICT(guild_id) DO UPDATE SET
                timezone = excluded.timezone,
                command_prefix = excluded.command_prefix,
                embed_color = excluded.embed_color,
                updated_at = unixepoch()",
        )
        .bind(guild_id)
        .bind(timezone)
        .bind(command_prefix)
        .bind(embed_color)
        .execute(db)
        .await?;

        Self::get_or_default(db, guild_id).await
    }

    /// Insert a default config row for `guild_id` if none exists, so foreign-key
    /// references (reminder configs, etc.) resolve. Existing rows are left
    /// untouched — never overwrites a guild's prefix, color, or timezone.
    pub async fn ensure_exists(db: &SqlitePool, guild_id: &str, timezone: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO guild_configs (guild_id, timezone) VALUES (?, ?)
             ON CONFLICT(guild_id) DO NOTHING",
        )
        .bind(guild_id)
        .bind(timezone)
        .execute(db)
        .await?;
        Ok(())
    }
}
