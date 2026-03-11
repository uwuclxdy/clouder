use crate::utils::parse_sqlite_datetime;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub user_id: String,
    pub timezone: String,
    pub dm_reminders_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserSettings {
    pub async fn get(pool: &SqlitePool, user_id: &str) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT user_id, timezone, dm_reminders_enabled, created_at, updated_at
            FROM user_settings
            WHERE user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Self {
                user_id: row.get("user_id"),
                timezone: row.get("timezone"),
                dm_reminders_enabled: row.get("dm_reminders_enabled"),
                created_at: parse_sqlite_datetime(&row.get::<String, _>("created_at")),
                updated_at: parse_sqlite_datetime(&row.get::<String, _>("updated_at")),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn upsert(
        pool: &SqlitePool,
        user_id: &str,
        timezone: &str,
        dm_reminders_enabled: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO user_settings (user_id, timezone, dm_reminders_enabled, updated_at)
            VALUES (?, ?, ?, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(user_id)
        .bind(timezone)
        .bind(dm_reminders_enabled)
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildConfig {
    pub guild_id: String,
    pub command_prefix: String,
    pub embed_color: Option<i64>,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl GuildConfig {
    pub async fn get(pool: &SqlitePool, guild_id: &str) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT guild_id, command_prefix, embed_color, timezone, created_at, updated_at
            FROM guild_configs
            WHERE guild_id = ?
            "#,
        )
        .bind(guild_id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Self {
                guild_id: row.get("guild_id"),
                command_prefix: row.get("command_prefix"),
                embed_color: row.get("embed_color"),
                timezone: row.get("timezone"),
                created_at: parse_sqlite_datetime(&row.get::<String, _>("created_at")),
                updated_at: parse_sqlite_datetime(&row.get::<String, _>("updated_at")),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn upsert(
        pool: &SqlitePool,
        guild_id: &str,
        command_prefix: &str,
        embed_color: Option<i64>,
        timezone: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO guild_configs (guild_id, command_prefix, embed_color, timezone, updated_at)
            VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(guild_id)
        .bind(command_prefix)
        .bind(embed_color)
        .bind(timezone)
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReminderType {
    Wysi,
    FemboyFriday,
    Custom,
}

impl ReminderType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Wysi => "wysi",
            Self::FemboyFriday => "femboy_friday",
            Self::Custom => "custom",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "wysi" => Some(Self::Wysi),
            "femboy_friday" => Some(Self::FemboyFriday),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderConfig {
    pub id: i64,
    pub guild_id: String,
    pub reminder_type: ReminderType,
    pub enabled: bool,
    pub channel_id: Option<String>,
    pub message_type: String,
    pub message_content: Option<String>,
    pub embed_title: Option<String>,
    pub embed_description: Option<String>,
    pub embed_color: Option<i64>,
    pub wysi_morning_time: Option<String>,
    pub wysi_evening_time: Option<String>,
    pub femboy_friday_time: Option<String>,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn parse_reminder_config_row(row: sqlx::sqlite::SqliteRow) -> ReminderConfig {
    let reminder_type_str: String = row.get("reminder_type");
    ReminderConfig {
        id: row.get("id"),
        guild_id: row.get("guild_id"),
        reminder_type: ReminderType::parse(&reminder_type_str).unwrap_or(ReminderType::Custom),
        enabled: row.get("enabled"),
        channel_id: row.get("channel_id"),
        message_type: row.get("message_type"),
        message_content: row.get("message_content"),
        embed_title: row.get("embed_title"),
        embed_description: row.get("embed_description"),
        embed_color: row.get("embed_color"),
        wysi_morning_time: row.get("wysi_morning_time"),
        wysi_evening_time: row.get("wysi_evening_time"),
        femboy_friday_time: row.get("femboy_friday_time"),
        timezone: row.get("timezone"),
        created_at: parse_sqlite_datetime(&row.get::<String, _>("created_at")),
        updated_at: parse_sqlite_datetime(&row.get::<String, _>("updated_at")),
    }
}

impl ReminderConfig {
    pub async fn get_by_guild(pool: &SqlitePool, guild_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, guild_id, reminder_type, enabled, channel_id, message_type,
                   message_content, embed_title, embed_description, embed_color,
                   wysi_morning_time, wysi_evening_time, femboy_friday_time,
                   timezone, created_at, updated_at
            FROM reminder_configs
            WHERE guild_id = ?
            "#,
        )
        .bind(guild_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(parse_reminder_config_row).collect())
    }

    pub async fn get_by_type(
        pool: &SqlitePool,
        guild_id: &str,
        reminder_type: &ReminderType,
    ) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, guild_id, reminder_type, enabled, channel_id, message_type,
                   message_content, embed_title, embed_description, embed_color,
                   wysi_morning_time, wysi_evening_time, femboy_friday_time,
                   timezone, created_at, updated_at
            FROM reminder_configs
            WHERE guild_id = ? AND reminder_type = ?
            "#,
        )
        .bind(guild_id)
        .bind(reminder_type.as_str())
        .fetch_optional(pool)
        .await?;

        Ok(row.map(parse_reminder_config_row))
    }

    pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, guild_id, reminder_type, enabled, channel_id, message_type,
                   message_content, embed_title, embed_description, embed_color,
                   wysi_morning_time, wysi_evening_time, femboy_friday_time,
                   timezone, created_at, updated_at
            FROM reminder_configs
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(parse_reminder_config_row))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert(
        pool: &SqlitePool,
        guild_id: &str,
        reminder_type: &ReminderType,
        channel_id: Option<&str>,
        message_type: &str,
        message_content: Option<&str>,
        embed_title: Option<&str>,
        embed_description: Option<&str>,
        embed_color: Option<i64>,
        wysi_morning_time: Option<&str>,
        wysi_evening_time: Option<&str>,
        femboy_friday_time: Option<&str>,
        timezone: &str,
    ) -> Result<i64, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT OR REPLACE INTO reminder_configs (
                guild_id, reminder_type, channel_id, message_type, message_content,
                embed_title, embed_description, embed_color,
                wysi_morning_time, wysi_evening_time, femboy_friday_time,
                timezone, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            RETURNING id
            "#,
        )
        .bind(guild_id)
        .bind(reminder_type.as_str())
        .bind(channel_id)
        .bind(message_type)
        .bind(message_content)
        .bind(embed_title)
        .bind(embed_description)
        .bind(embed_color)
        .bind(wysi_morning_time)
        .bind(wysi_evening_time)
        .bind(femboy_friday_time)
        .bind(timezone)
        .fetch_one(pool)
        .await?;

        Ok(row.get("id"))
    }

    pub async fn set_enabled(pool: &SqlitePool, id: i64, enabled: bool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE reminder_configs
            SET enabled = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(enabled)
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM reminder_configs
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderPingRole {
    pub id: i64,
    pub config_id: i64,
    pub role_id: String,
}

impl ReminderPingRole {
    pub async fn get_by_config(
        pool: &SqlitePool,
        config_id: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, config_id, role_id
            FROM reminder_ping_roles
            WHERE config_id = ?
            "#,
        )
        .bind(config_id)
        .fetch_all(pool)
        .await?;

        let mut roles = Vec::new();
        for row in rows {
            roles.push(Self {
                id: row.get("id"),
                config_id: row.get("config_id"),
                role_id: row.get("role_id"),
            });
        }

        Ok(roles)
    }

    pub async fn set_roles(
        pool: &SqlitePool,
        config_id: i64,
        role_ids: &[String],
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM reminder_ping_roles
            WHERE config_id = ?
            "#,
        )
        .bind(config_id)
        .execute(pool)
        .await?;

        for role_id in role_ids {
            sqlx::query(
                r#"
                INSERT INTO reminder_ping_roles (config_id, role_id)
                VALUES (?, ?)
                "#,
            )
            .bind(config_id)
            .bind(role_id)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    pub async fn delete_by_config(pool: &SqlitePool, config_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM reminder_ping_roles
            WHERE config_id = ?
            "#,
        )
        .bind(config_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderSubscription {
    pub id: i64,
    pub user_id: String,
    pub config_id: i64,
    pub subscribed_at: DateTime<Utc>,
}

impl ReminderSubscription {
    pub async fn get_by_user(pool: &SqlitePool, user_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, config_id, subscribed_at
            FROM reminder_subscriptions
            WHERE user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut subscriptions = Vec::new();
        for row in rows {
            subscriptions.push(Self {
                id: row.get("id"),
                user_id: row.get("user_id"),
                config_id: row.get("config_id"),
                subscribed_at: parse_sqlite_datetime(&row.get::<String, _>("subscribed_at")),
            });
        }

        Ok(subscriptions)
    }

    pub async fn get_by_config(
        pool: &SqlitePool,
        config_id: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, config_id, subscribed_at
            FROM reminder_subscriptions
            WHERE config_id = ?
            "#,
        )
        .bind(config_id)
        .fetch_all(pool)
        .await?;

        let mut subscriptions = Vec::new();
        for row in rows {
            subscriptions.push(Self {
                id: row.get("id"),
                user_id: row.get("user_id"),
                config_id: row.get("config_id"),
                subscribed_at: parse_sqlite_datetime(&row.get::<String, _>("subscribed_at")),
            });
        }

        Ok(subscriptions)
    }

    pub async fn subscribe(
        pool: &SqlitePool,
        user_id: &str,
        config_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO reminder_subscriptions (user_id, config_id)
            VALUES (?, ?)
            "#,
        )
        .bind(user_id)
        .bind(config_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn unsubscribe(
        pool: &SqlitePool,
        user_id: &str,
        config_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM reminder_subscriptions
            WHERE user_id = ? AND config_id = ?
            "#,
        )
        .bind(user_id)
        .bind(config_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn unsubscribe_all_for_user(
        pool: &SqlitePool,
        user_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM reminder_subscriptions
            WHERE user_id = ?
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_by_id(pool: &SqlitePool, sub_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM reminder_subscriptions
            WHERE id = ?
            "#,
        )
        .bind(sub_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderLog {
    pub id: i64,
    pub config_id: i64,
    pub execution_time: DateTime<Utc>,
    pub status: String,
    pub error_message: Option<String>,
    pub channel_sent: bool,
    pub dm_count: i64,
    pub dm_failed_count: i64,
    pub created_at: DateTime<Utc>,
}

impl ReminderLog {
    pub async fn create(
        pool: &SqlitePool,
        config_id: i64,
        status: &str,
        error_message: Option<&str>,
        channel_sent: bool,
        dm_count: i64,
        dm_failed_count: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO reminder_logs (config_id, execution_time, status, error_message, channel_sent, dm_count, dm_failed_count)
            VALUES (?, CURRENT_TIMESTAMP, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(config_id)
        .bind(status)
        .bind(error_message)
        .bind(channel_sent)
        .bind(dm_count)
        .bind(dm_failed_count)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_recent_by_config(
        pool: &SqlitePool,
        config_id: i64,
        limit: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, config_id, execution_time, status, error_message,
                   channel_sent, dm_count, dm_failed_count, created_at
            FROM reminder_logs
            WHERE config_id = ?
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(config_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        let mut logs = Vec::new();
        for row in rows {
            logs.push(Self {
                id: row.get("id"),
                config_id: row.get("config_id"),
                execution_time: parse_sqlite_datetime(&row.get::<String, _>("execution_time")),
                status: row.get("status"),
                error_message: row.get("error_message"),
                channel_sent: row.get("channel_sent"),
                dm_count: row.get("dm_count"),
                dm_failed_count: row.get("dm_failed_count"),
                created_at: parse_sqlite_datetime(&row.get::<String, _>("created_at")),
            });
        }

        Ok(logs)
    }
}
