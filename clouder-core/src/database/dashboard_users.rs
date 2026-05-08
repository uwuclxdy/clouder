use anyhow::Result;
use hmac::{Hmac, KeyInit, Mac};
use rand::Rng;
use sha2::Sha256;
use sqlx::SqlitePool;
use subtle::ConstantTimeEq;

// 32 random bytes (256 bits) is enough entropy that the public hex form
// (64 chars) won't collide and isn't brute-forceable, while staying short
// enough for users to copy-paste comfortably.
const API_KEY_BYTES: usize = 32;

#[derive(Debug, sqlx::FromRow)]
pub struct DashboardUser {
    pub user_id: String,
    pub api_key_hash: Option<String>,
    pub oauth_token: Option<String>,
    pub oauth_token_updated_at: Option<i64>,
    pub username: Option<String>,
    pub avatar: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

fn generate_api_key() -> String {
    let mut buf = [0u8; API_KEY_BYTES];
    rand::rng().fill_bytes(&mut buf);
    buf.iter().map(|b| format!("{:02x}", b)).collect()
}

/// HMAC-SHA256 of the API key using the server-side pepper, hex-encoded.
/// Lookup compares the stored hash against the hash of the submitted key,
/// so a database dump never reveals usable credentials.
pub fn hash_api_key(pepper: &str, key: &str) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(pepper.as_bytes()).expect("HMAC accepts any key length");
    mac.update(key.as_bytes());
    let bytes = mac.finalize().into_bytes();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

impl DashboardUser {
    /// Idempotently provisions a dashboard row for `user_id`. Returns the row
    /// plus the plaintext key only when a fresh key was minted; subsequent
    /// calls return an empty string because the plaintext is unrecoverable.
    ///
    /// Three states matter:
    ///   1. No row exists → INSERT a fresh row with a new hash.
    ///   2. Row exists but `api_key_hash` is NULL (user predates the security
    ///      migration, or hash was wiped by migration 010) → INSERT collides;
    ///      ON CONFLICT backfills the hash. The key is returned to the caller.
    ///   3. Row exists with a hash → early return, no SQL touched, no new key.
    pub async fn upsert(db: &SqlitePool, user_id: &str, pepper: &str) -> Result<(Self, String)> {
        if let Some(existing) = Self::get_by_user_id(db, user_id).await?
            && existing.api_key_hash.is_some()
        {
            return Ok((existing, String::new()));
        }
        let key = generate_api_key();
        let hash = hash_api_key(pepper, &key);
        sqlx::query(
            "INSERT INTO dashboard_users (user_id, api_key_hash) VALUES (?, ?)
             ON CONFLICT(user_id) DO UPDATE SET api_key_hash = excluded.api_key_hash, updated_at = unixepoch()
             WHERE dashboard_users.api_key_hash IS NULL",
        )
        .bind(user_id)
        .bind(&hash)
        .execute(db)
        .await?;

        let user = Self::get_by_user_id(db, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("failed to upsert dashboard user"))?;
        Ok((user, key))
    }

    pub async fn regenerate_key(db: &SqlitePool, user_id: &str, pepper: &str) -> Result<String> {
        let key = generate_api_key();
        let hash = hash_api_key(pepper, &key);
        sqlx::query(
            "UPDATE dashboard_users SET api_key_hash = ?, updated_at = unixepoch() WHERE user_id = ?",
        )
        .bind(&hash)
        .bind(user_id)
        .execute(db)
        .await?;
        Ok(key)
    }

    pub async fn get_by_user_id(db: &SqlitePool, user_id: &str) -> Result<Option<Self>> {
        Ok(
            sqlx::query_as::<_, Self>("SELECT * FROM dashboard_users WHERE user_id = ?")
                .bind(user_id)
                .fetch_optional(db)
                .await?,
        )
    }

    /// Looks up a user by submitted API key. Hashes the key with the pepper and
    /// performs a constant-time comparison against the stored hash to prevent
    /// timing-based key recovery.
    pub async fn get_by_api_key(
        db: &SqlitePool,
        pepper: &str,
        api_key: &str,
    ) -> Result<Option<Self>> {
        let hash = hash_api_key(pepper, api_key);
        let candidate =
            sqlx::query_as::<_, Self>("SELECT * FROM dashboard_users WHERE api_key_hash = ?")
                .bind(&hash)
                .fetch_optional(db)
                .await?;

        Ok(candidate.filter(|u| {
            u.api_key_hash
                .as_deref()
                .map(|stored| stored.as_bytes().ct_eq(hash.as_bytes()).into())
                .unwrap_or(false)
        }))
    }

    pub async fn store_oauth_token(db: &SqlitePool, user_id: &str, token: &str) -> Result<()> {
        sqlx::query(
            "UPDATE dashboard_users
             SET oauth_token = ?, oauth_token_updated_at = unixepoch(), updated_at = unixepoch()
             WHERE user_id = ?",
        )
        .bind(token)
        .bind(user_id)
        .execute(db)
        .await?;
        Ok(())
    }

    pub async fn clear_oauth_token(db: &SqlitePool, user_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE dashboard_users SET oauth_token = NULL, oauth_token_updated_at = NULL WHERE user_id = ?",
        )
        .bind(user_id)
        .execute(db)
        .await?;
        Ok(())
    }

    pub async fn store_profile(
        db: &SqlitePool,
        user_id: &str,
        username: &str,
        avatar: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE dashboard_users
             SET username = ?, avatar = ?, updated_at = unixepoch()
             WHERE user_id = ?",
        )
        .bind(username)
        .bind(avatar)
        .bind(user_id)
        .execute(db)
        .await?;
        Ok(())
    }
}
