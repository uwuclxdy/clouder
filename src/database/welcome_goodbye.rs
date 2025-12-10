use crate::utils::parse_sqlite_datetime;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelcomeGoodbyeConfig {
    pub guild_id: String,
    pub welcome_enabled: bool,
    pub goodbye_enabled: bool,
    pub welcome_channel_id: Option<String>,
    pub goodbye_channel_id: Option<String>,
    pub welcome_message_type: String,
    pub goodbye_message_type: String,
    pub welcome_message_content: Option<String>,
    pub goodbye_message_content: Option<String>,
    // Welcome embed fields
    pub welcome_embed_title: Option<String>,
    pub welcome_embed_description: Option<String>,
    pub welcome_embed_color: Option<i32>,
    pub welcome_embed_footer: Option<String>,
    pub welcome_embed_thumbnail: Option<String>,
    pub welcome_embed_image: Option<String>,
    pub welcome_embed_timestamp: bool,
    // Goodbye embed fields
    pub goodbye_embed_title: Option<String>,
    pub goodbye_embed_description: Option<String>,
    pub goodbye_embed_color: Option<i32>,
    pub goodbye_embed_footer: Option<String>,
    pub goodbye_embed_thumbnail: Option<String>,
    pub goodbye_embed_image: Option<String>,
    pub goodbye_embed_timestamp: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for WelcomeGoodbyeConfig {
    fn default() -> Self {
        Self {
            guild_id: String::new(),
            welcome_enabled: false,
            goodbye_enabled: false,
            welcome_channel_id: None,
            goodbye_channel_id: None,
            welcome_message_type: "embed".to_string(),
            goodbye_message_type: "embed".to_string(),
            welcome_message_content: None,
            goodbye_message_content: None,
            welcome_embed_title: None,
            welcome_embed_description: None,
            welcome_embed_color: None,
            welcome_embed_footer: None,
            welcome_embed_thumbnail: None,
            welcome_embed_image: None,
            welcome_embed_timestamp: false,
            goodbye_embed_title: None,
            goodbye_embed_description: None,
            goodbye_embed_color: None,
            goodbye_embed_footer: None,
            goodbye_embed_thumbnail: None,
            goodbye_embed_image: None,
            goodbye_embed_timestamp: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

impl WelcomeGoodbyeConfig {
    pub async fn get_config(
        pool: &SqlitePool,
        guild_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT guild_id, welcome_enabled, goodbye_enabled, welcome_channel_id, goodbye_channel_id,
                   welcome_message_type, goodbye_message_type, welcome_message_content, goodbye_message_content,
                   welcome_embed_title, welcome_embed_description, welcome_embed_color, welcome_embed_footer,
                   welcome_embed_thumbnail, welcome_embed_image, welcome_embed_timestamp,
                   goodbye_embed_title, goodbye_embed_description, goodbye_embed_color, goodbye_embed_footer,
                   goodbye_embed_thumbnail, goodbye_embed_image, goodbye_embed_timestamp,
                   created_at, updated_at
            FROM welcome_goodbye_configs
            WHERE guild_id = ?
            "#,
        )
        .bind(guild_id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Self {
                guild_id: row.get("guild_id"),
                welcome_enabled: row.get("welcome_enabled"),
                goodbye_enabled: row.get("goodbye_enabled"),
                welcome_channel_id: row.get("welcome_channel_id"),
                goodbye_channel_id: row.get("goodbye_channel_id"),
                welcome_message_type: row.get("welcome_message_type"),
                goodbye_message_type: row.get("goodbye_message_type"),
                welcome_message_content: row.get("welcome_message_content"),
                goodbye_message_content: row.get("goodbye_message_content"),
                welcome_embed_title: row.get("welcome_embed_title"),
                welcome_embed_description: row.get("welcome_embed_description"),
                welcome_embed_color: row.get("welcome_embed_color"),
                welcome_embed_footer: row.get("welcome_embed_footer"),
                welcome_embed_thumbnail: row.get("welcome_embed_thumbnail"),
                welcome_embed_image: row.get("welcome_embed_image"),
                welcome_embed_timestamp: row.get("welcome_embed_timestamp"),
                goodbye_embed_title: row.get("goodbye_embed_title"),
                goodbye_embed_description: row.get("goodbye_embed_description"),
                goodbye_embed_color: row.get("goodbye_embed_color"),
                goodbye_embed_footer: row.get("goodbye_embed_footer"),
                goodbye_embed_thumbnail: row.get("goodbye_embed_thumbnail"),
                goodbye_embed_image: row.get("goodbye_embed_image"),
                goodbye_embed_timestamp: row.get("goodbye_embed_timestamp"),
                created_at: parse_sqlite_datetime(&row.get::<String, _>("created_at")),
                updated_at: parse_sqlite_datetime(&row.get::<String, _>("updated_at")),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn upsert_config(pool: &SqlitePool, config: &Self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO welcome_goodbye_configs (
                guild_id, welcome_enabled, goodbye_enabled, welcome_channel_id, goodbye_channel_id,
                welcome_message_type, goodbye_message_type, welcome_message_content, goodbye_message_content,
                welcome_embed_title, welcome_embed_description, welcome_embed_color, welcome_embed_footer,
                welcome_embed_thumbnail, welcome_embed_image, welcome_embed_timestamp,
                goodbye_embed_title, goodbye_embed_description, goodbye_embed_color, goodbye_embed_footer,
                goodbye_embed_thumbnail, goodbye_embed_image, goodbye_embed_timestamp,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                     COALESCE((SELECT created_at FROM welcome_goodbye_configs WHERE guild_id = ?), CURRENT_TIMESTAMP),
                     CURRENT_TIMESTAMP)
            "#,
        )
        .bind(&config.guild_id)
        .bind(config.welcome_enabled)
        .bind(config.goodbye_enabled)
        .bind(&config.welcome_channel_id)
        .bind(&config.goodbye_channel_id)
        .bind(&config.welcome_message_type)
        .bind(&config.goodbye_message_type)
        .bind(&config.welcome_message_content)
        .bind(&config.goodbye_message_content)
        .bind(&config.welcome_embed_title)
        .bind(&config.welcome_embed_description)
        .bind(config.welcome_embed_color)
        .bind(&config.welcome_embed_footer)
        .bind(&config.welcome_embed_thumbnail)
        .bind(&config.welcome_embed_image)
        .bind(config.welcome_embed_timestamp)
        .bind(&config.goodbye_embed_title)
        .bind(&config.goodbye_embed_description)
        .bind(config.goodbye_embed_color)
        .bind(&config.goodbye_embed_footer)
        .bind(&config.goodbye_embed_thumbnail)
        .bind(&config.goodbye_embed_image)
        .bind(config.goodbye_embed_timestamp)
        .bind(&config.guild_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

pub fn get_member_placeholders(
    user: &serenity::model::user::User,
    guild_name: &str,
    member_count: u64,
    member: Option<&serenity::model::guild::Member>,
) -> HashMap<String, String> {
    let mut placeholders = HashMap::new();

    placeholders.insert("user".to_string(), format!("<@{}>", user.id));
    placeholders.insert("username".to_string(), user.name.clone());
    placeholders.insert("server".to_string(), guild_name.to_string());
    placeholders.insert("member_count".to_string(), member_count.to_string());
    placeholders.insert("user_id".to_string(), user.id.to_string());

    if let Some(member) = member {
        if let Some(joined_at) = member.joined_at {
            placeholders.insert(
                "join_date".to_string(),
                joined_at.format("%Y-%m-%d").to_string(),
            );
        }
    } else {
        // For welcome messages, use account creation date if member info not available
        let created_at = user.created_at();
        placeholders.insert(
            "join_date".to_string(),
            created_at.format("%Y-%m-%d").to_string(),
        );
    }

    placeholders
}
