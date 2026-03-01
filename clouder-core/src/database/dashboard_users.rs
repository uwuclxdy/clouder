use anyhow::Result;
use rand::Rng;
use sqlx::SqlitePool;

#[derive(Debug, sqlx::FromRow)]
pub struct DashboardUser {
    pub user_id: String,
    pub api_key: String,
    pub created_at: i64,
    pub updated_at: i64,
}

fn generate_api_key() -> String {
    let mut buf = [0u8; 32];
    rand::rng().fill_bytes(&mut buf);
    buf.iter().map(|b| format!("{:02x}", b)).collect()
}

impl DashboardUser {
    pub async fn upsert(db: &SqlitePool, user_id: &str) -> Result<Self> {
        let key = generate_api_key();
        sqlx::query("INSERT OR IGNORE INTO dashboard_users (user_id, api_key) VALUES (?, ?)")
            .bind(user_id)
            .bind(&key)
            .execute(db)
            .await?;

        Self::get_by_user_id(db, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("failed to upsert dashboard user"))
    }

    pub async fn regenerate_key(db: &SqlitePool, user_id: &str) -> Result<Self> {
        let key = generate_api_key();
        sqlx::query(
            "UPDATE dashboard_users SET api_key = ?, updated_at = unixepoch() WHERE user_id = ?",
        )
        .bind(&key)
        .bind(user_id)
        .execute(db)
        .await?;

        Self::get_by_user_id(db, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("dashboard user not found"))
    }

    pub async fn get_by_user_id(db: &SqlitePool, user_id: &str) -> Result<Option<Self>> {
        Ok(
            sqlx::query_as::<_, Self>("SELECT * FROM dashboard_users WHERE user_id = ?")
                .bind(user_id)
                .fetch_optional(db)
                .await?,
        )
    }

    pub async fn get_by_api_key(db: &SqlitePool, api_key: &str) -> Result<Option<Self>> {
        Ok(
            sqlx::query_as::<_, Self>("SELECT * FROM dashboard_users WHERE api_key = ?")
                .bind(api_key)
                .fetch_optional(db)
                .await?,
        )
    }
}
