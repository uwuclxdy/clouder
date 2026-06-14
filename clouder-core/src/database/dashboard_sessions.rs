use crate::crypto::random_hex;
use anyhow::Result;
use sqlx::SqlitePool;
use subtle::ConstantTimeEq;

// 32 random bytes (256 bits) gives both opaque session IDs and CSRF tokens
// well past the brute-force threshold even at internet-scale request rates.
const SESSION_ID_BYTES: usize = 32;
const CSRF_TOKEN_BYTES: usize = 32;

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct DashboardSession {
    pub session_id: String,
    pub user_id: String,
    pub csrf_token: String,
    pub expires_at: i64,
    pub created_at: i64,
}

impl DashboardSession {
    pub async fn create(db: &SqlitePool, user_id: &str, ttl_seconds: i64) -> Result<Self> {
        let session_id = random_hex(SESSION_ID_BYTES);
        let csrf_token = random_hex(CSRF_TOKEN_BYTES);
        let expires_at = chrono::Utc::now().timestamp() + ttl_seconds;

        sqlx::query(
            "INSERT INTO dashboard_sessions (session_id, user_id, csrf_token, expires_at) VALUES (?, ?, ?, ?)",
        )
        .bind(&session_id)
        .bind(user_id)
        .bind(&csrf_token)
        .bind(expires_at)
        .execute(db)
        .await?;

        Ok(Self {
            session_id,
            user_id: user_id.to_string(),
            csrf_token,
            expires_at,
            created_at: chrono::Utc::now().timestamp(),
        })
    }

    /// Looks up a session, validating expiry. Returns `None` for missing,
    /// expired, or otherwise unusable sessions — never reveals which.
    pub async fn get_active(db: &SqlitePool, session_id: &str) -> Result<Option<Self>> {
        let row: Option<Self> = sqlx::query_as(
            "SELECT * FROM dashboard_sessions WHERE session_id = ? AND expires_at > unixepoch()",
        )
        .bind(session_id)
        .fetch_optional(db)
        .await?;
        Ok(row)
    }

    pub async fn delete(db: &SqlitePool, session_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM dashboard_sessions WHERE session_id = ?")
            .bind(session_id)
            .execute(db)
            .await?;
        Ok(())
    }

    pub async fn delete_expired(db: &SqlitePool) -> Result<u64> {
        let result = sqlx::query("DELETE FROM dashboard_sessions WHERE expires_at <= unixepoch()")
            .execute(db)
            .await?;
        Ok(result.rows_affected())
    }

    pub fn csrf_matches(&self, presented: &str) -> bool {
        self.csrf_token
            .as_bytes()
            .ct_eq(presented.as_bytes())
            .into()
    }
}
