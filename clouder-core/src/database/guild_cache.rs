use anyhow::Result;
use sqlx::SqlitePool;

#[derive(Debug, sqlx::FromRow)]
pub struct CachedGuild {
    pub user_id: String,
    pub guild_id: String,
    pub name: String,
    pub icon: Option<String>,
    pub permissions: i64,
    pub updated_at: i64,
}

impl CachedGuild {
    pub async fn get_for_user(pool: &SqlitePool, user_id: &str) -> Result<Vec<Self>> {
        Ok(sqlx::query_as::<_, Self>(
            "SELECT * FROM user_guild_cache WHERE user_id = ? ORDER BY name ASC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?)
    }

    pub async fn get_name(pool: &SqlitePool, user_id: &str, guild_id: &str) -> Option<String> {
        sqlx::query_scalar::<_, String>(
            "SELECT name FROM user_guild_cache WHERE user_id = ? AND guild_id = ?",
        )
        .bind(user_id)
        .bind(guild_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
    }

    pub async fn user_has_guild(pool: &SqlitePool, user_id: &str, guild_id: &str) -> Result<bool> {
        Ok(sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM user_guild_cache WHERE user_id = ? AND guild_id = ?)",
        )
        .bind(user_id)
        .bind(guild_id)
        .fetch_one(pool)
        .await?)
    }

    pub async fn get_user_permissions(
        pool: &SqlitePool,
        user_id: &str,
        guild_id: &str,
    ) -> Result<Option<i64>> {
        Ok(sqlx::query_scalar::<_, i64>(
            "SELECT permissions FROM user_guild_cache WHERE user_id = ? AND guild_id = ?",
        )
        .bind(user_id)
        .bind(guild_id)
        .fetch_optional(pool)
        .await?)
    }

    pub async fn replace_for_user(
        pool: &SqlitePool,
        user_id: &str,
        guilds: &[(String, String, Option<String>, i64)],
    ) -> Result<()> {
        let mut tx = pool.begin().await?;
        sqlx::query("DELETE FROM user_guild_cache WHERE user_id = ?")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        for (guild_id, name, icon, permissions) in guilds {
            sqlx::query(
                "INSERT INTO user_guild_cache (user_id, guild_id, name, icon, permissions) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(user_id)
            .bind(guild_id)
            .bind(name)
            .bind(icon)
            .bind(permissions)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}
