use anyhow::Result;
use sqlx::SqlitePool;

#[derive(Debug, sqlx::FromRow)]
pub struct CachedGuild {
    pub user_id: String,
    pub guild_id: String,
    pub name: String,
    pub icon: Option<String>,
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

    pub async fn replace_for_user(
        pool: &SqlitePool,
        user_id: &str,
        guilds: &[(String, String, Option<String>)],
    ) -> Result<()> {
        let mut tx = pool.begin().await?;
        sqlx::query("DELETE FROM user_guild_cache WHERE user_id = ?")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        for (guild_id, name, icon) in guilds {
            sqlx::query(
                "INSERT INTO user_guild_cache (user_id, guild_id, name, icon) VALUES (?, ?, ?, ?)",
            )
            .bind(user_id)
            .bind(guild_id)
            .bind(name)
            .bind(icon)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}
