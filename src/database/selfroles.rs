use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SelfRoleConfig {
    pub id: i64,
    pub guild_id: String,
    pub channel_id: String,
    pub message_id: Option<String>,
    pub title: String,
    pub body: String,
    pub selection_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SelfRoleRole {
    pub id: i64,
    pub config_id: i64,
    pub role_id: String,
    pub emoji: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct SelfRoleCooldown {
    pub user_id: String,
    pub role_id: String,
    pub guild_id: String,
    pub expires_at: DateTime<Utc>,
}

impl SelfRoleConfig {
    pub async fn create(
        pool: &SqlitePool,
        guild_id: &str,
        channel_id: &str,
        title: &str,
        body: &str,
        selection_type: &str,
    ) -> Result<Self> {
        let result = sqlx::query(
            r#"
            INSERT INTO selfrole_configs (guild_id, channel_id, title, body, selection_type)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(guild_id)
        .bind(channel_id)
        .bind(title)
        .bind(body)
        .bind(selection_type)
        .execute(pool)
        .await?;
        
        let config = sqlx::query_as::<_, Self>(
            "SELECT * FROM selfrole_configs WHERE id = ?"
        )
        .bind(result.last_insert_rowid())
        .fetch_one(pool)
        .await?;
        
        Ok(config)
    }
    
    pub async fn update_message_id(
        &mut self,
        pool: &SqlitePool,
        message_id: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE selfrole_configs 
            SET message_id = ?, updated_at = CURRENT_TIMESTAMP 
            WHERE id = ?
            "#
        )
        .bind(message_id)
        .bind(self.id)
        .execute(pool)
        .await?;
        
        self.message_id = Some(message_id.to_string());
        Ok(())
    }
    
    pub async fn get_by_message_id(
        pool: &SqlitePool,
        message_id: &str,
    ) -> Result<Option<Self>> {
        let config = sqlx::query_as::<_, Self>(
            "SELECT * FROM selfrole_configs WHERE message_id = ?"
        )
        .bind(message_id)
        .fetch_optional(pool)
        .await?;
        
        Ok(config)
    }
    
    pub async fn get_by_guild(
        pool: &SqlitePool,
        guild_id: &str,
    ) -> Result<Vec<Self>> {
        let configs = sqlx::query_as::<_, Self>(
            "SELECT * FROM selfrole_configs WHERE guild_id = ? ORDER BY created_at DESC"
        )
        .bind(guild_id)
        .fetch_all(pool)
        .await?;
        
        Ok(configs)
    }
    
    pub async fn delete(&self, pool: &SqlitePool) -> Result<()> {
        sqlx::query("DELETE FROM selfrole_configs WHERE id = ?")
            .bind(self.id)
            .execute(pool)
            .await?;
        
        Ok(())
    }
    
    pub async fn update(
        &mut self,
        pool: &SqlitePool,
        title: &str,
        body: &str,
        selection_type: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE selfrole_configs 
            SET title = ?, body = ?, selection_type = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#
        )
        .bind(title)
        .bind(body)
        .bind(selection_type)
        .bind(self.id)
        .execute(pool)
        .await?;
        
        // Update the local instance
        self.title = title.to_string();
        self.body = body.to_string();
        self.selection_type = selection_type.to_string();
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    pub async fn get_roles(&self, pool: &SqlitePool) -> Result<Vec<SelfRoleRole>> {
        let roles = sqlx::query_as::<_, SelfRoleRole>(
            "SELECT * FROM selfrole_roles WHERE config_id = ?"
        )
        .bind(self.id)
        .fetch_all(pool)
        .await?;
        
        Ok(roles)
    }
    
    pub async fn delete_by_message_id(
        pool: &SqlitePool,
        message_id: &str,
    ) -> Result<bool> {
        let result = sqlx::query("DELETE FROM selfrole_configs WHERE message_id = ?")
            .bind(message_id)
            .execute(pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    pub async fn get_by_guild_id(
        pool: &SqlitePool,
        guild_id: u64,
    ) -> Result<Vec<Self>> {
        let configs = sqlx::query_as::<_, Self>(
            "SELECT * FROM selfrole_configs WHERE guild_id = ? ORDER BY created_at DESC"
        )
        .bind(guild_id.to_string())
        .fetch_all(pool)
        .await?;
        
        Ok(configs)
    }
    
    pub async fn get_by_message_id_u64(
        pool: &SqlitePool,
        message_id: u64,
    ) -> Result<Option<Self>> {
        let config = sqlx::query_as::<_, Self>(
            "SELECT * FROM selfrole_configs WHERE message_id = ?"
        )
        .bind(message_id.to_string())
        .fetch_optional(pool)
        .await?;
        
        Ok(config)
    }
}

impl SelfRoleRole {
    pub async fn create(
        pool: &SqlitePool,
        config_id: i64,
        role_id: &str,
        emoji: &str,
    ) -> Result<Self> {
        let result = sqlx::query(
            r#"
            INSERT INTO selfrole_roles (config_id, role_id, emoji)
            VALUES (?, ?, ?)
            "#
        )
        .bind(config_id)
        .bind(role_id)
        .bind(emoji)
        .execute(pool)
        .await?;
        
        let role = sqlx::query_as::<_, Self>(
            "SELECT * FROM selfrole_roles WHERE id = ?"
        )
        .bind(result.last_insert_rowid())
        .fetch_one(pool)
        .await?;
        
        Ok(role)
    }
    
    pub async fn delete_by_config_id(
        pool: &SqlitePool,
        config_id: i64,
    ) -> Result<()> {
        sqlx::query("DELETE FROM selfrole_roles WHERE config_id = ?")
            .bind(config_id)
            .execute(pool)
            .await?;
        
        Ok(())
    }
}

impl SelfRoleCooldown {
    pub async fn create(
        pool: &SqlitePool,
        user_id: &str,
        role_id: &str,
        guild_id: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO selfrole_cooldowns (user_id, role_id, guild_id, expires_at)
            VALUES (?, ?, ?, ?)
            "#
        )
        .bind(user_id)
        .bind(role_id)
        .bind(guild_id)
        .bind(expires_at)
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn check_cooldown(
        pool: &SqlitePool,
        user_id: &str,
        role_id: &str,
        guild_id: &str,
    ) -> Result<bool> {
        let now = Utc::now();
        
        let cooldown = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM selfrole_cooldowns 
            WHERE user_id = ? AND role_id = ? AND guild_id = ? AND expires_at > ?
            "#
        )
        .bind(user_id)
        .bind(role_id)
        .bind(guild_id)
        .bind(now)
        .fetch_optional(pool)
        .await?;
        
        Ok(cooldown.is_some())
    }
    
    pub async fn cleanup_expired(pool: &SqlitePool) -> Result<()> {
        let now = Utc::now();
        
        sqlx::query("DELETE FROM selfrole_cooldowns WHERE expires_at <= ?")
            .bind(now)
            .execute(pool)
            .await?;
        
        Ok(())
    }
}