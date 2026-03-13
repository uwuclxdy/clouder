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
    Custom,
}

impl ReminderType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Wysi => "wysi",
            Self::Custom => "custom",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "wysi" => Some(Self::Wysi),
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
                   wysi_morning_time, wysi_evening_time,
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
                   wysi_morning_time, wysi_evening_time,
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
                   wysi_morning_time, wysi_evening_time,
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
        timezone: &str,
    ) -> Result<i64, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO reminder_configs (
                guild_id, reminder_type, channel_id, message_type, message_content,
                embed_title, embed_description, embed_color,
                wysi_morning_time, wysi_evening_time,
                timezone, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(guild_id, reminder_type) DO UPDATE SET
                channel_id = excluded.channel_id,
                message_type = excluded.message_type,
                message_content = excluded.message_content,
                embed_title = excluded.embed_title,
                embed_description = excluded.embed_description,
                embed_color = excluded.embed_color,
                wysi_morning_time = excluded.wysi_morning_time,
                wysi_evening_time = excluded.wysi_evening_time,
                timezone = excluded.timezone,
                updated_at = CURRENT_TIMESTAMP
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
        let mut tx = pool.begin().await?;

        sqlx::query("DELETE FROM reminder_ping_roles WHERE config_id = ?")
            .bind(config_id)
            .execute(&mut *tx)
            .await?;

        for role_id in role_ids {
            sqlx::query("INSERT INTO reminder_ping_roles (config_id, role_id) VALUES (?, ?)")
                .bind(config_id)
                .bind(role_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
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

// ---- Custom reminders ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomReminder {
    pub id: i64,
    pub guild_id: String,
    pub name: String,
    pub enabled: bool,
    pub channel_id: Option<String>,
    pub schedule_time: String,
    pub schedule_days: String,
    pub timezone: String,
    pub message_type: String,
    pub message_content: Option<String>,
    pub embed_title: Option<String>,
    pub embed_description: Option<String>,
    pub embed_color: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn parse_custom_reminder_row(row: sqlx::sqlite::SqliteRow) -> CustomReminder {
    CustomReminder {
        id: row.get("id"),
        guild_id: row.get("guild_id"),
        name: row.get("name"),
        enabled: row.get("enabled"),
        channel_id: row.get("channel_id"),
        schedule_time: row.get("schedule_time"),
        schedule_days: row.get("schedule_days"),
        timezone: row.get("timezone"),
        message_type: row.get("message_type"),
        message_content: row.get("message_content"),
        embed_title: row.get("embed_title"),
        embed_description: row.get("embed_description"),
        embed_color: row.get("embed_color"),
        created_at: parse_sqlite_datetime(&row.get::<String, _>("created_at")),
        updated_at: parse_sqlite_datetime(&row.get::<String, _>("updated_at")),
    }
}

impl CustomReminder {
    pub async fn get_by_guild(pool: &SqlitePool, guild_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, guild_id, name, enabled, channel_id, schedule_time, schedule_days,
                    timezone, message_type, message_content, embed_title, embed_description,
                    embed_color, created_at, updated_at
             FROM custom_reminders WHERE guild_id = ? ORDER BY id",
        )
        .bind(guild_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(parse_custom_reminder_row).collect())
    }

    pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, guild_id, name, enabled, channel_id, schedule_time, schedule_days,
                    timezone, message_type, message_content, embed_title, embed_description,
                    embed_color, created_at, updated_at
             FROM custom_reminders WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(parse_custom_reminder_row))
    }

    pub async fn count_by_guild(pool: &SqlitePool, guild_id: &str) -> Result<i64, sqlx::Error> {
        let row =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM custom_reminders WHERE guild_id = ?")
                .bind(guild_id)
                .fetch_one(pool)
                .await?;
        Ok(row.0)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        pool: &SqlitePool,
        guild_id: &str,
        name: &str,
        channel_id: Option<&str>,
        schedule_time: &str,
        schedule_days: &str,
        timezone: &str,
        message_type: &str,
        message_content: Option<&str>,
        embed_title: Option<&str>,
        embed_description: Option<&str>,
        embed_color: Option<i64>,
    ) -> Result<i64, sqlx::Error> {
        let row = sqlx::query(
            "INSERT INTO custom_reminders (
                guild_id, name, channel_id, schedule_time, schedule_days, timezone,
                message_type, message_content, embed_title, embed_description, embed_color
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             RETURNING id",
        )
        .bind(guild_id)
        .bind(name)
        .bind(channel_id)
        .bind(schedule_time)
        .bind(schedule_days)
        .bind(timezone)
        .bind(message_type)
        .bind(message_content)
        .bind(embed_title)
        .bind(embed_description)
        .bind(embed_color)
        .fetch_one(pool)
        .await?;

        Ok(row.get("id"))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        name: &str,
        channel_id: Option<&str>,
        schedule_time: &str,
        schedule_days: &str,
        timezone: &str,
        message_type: &str,
        message_content: Option<&str>,
        embed_title: Option<&str>,
        embed_description: Option<&str>,
        embed_color: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE custom_reminders SET
                name = ?, channel_id = ?, schedule_time = ?, schedule_days = ?,
                timezone = ?, message_type = ?, message_content = ?,
                embed_title = ?, embed_description = ?, embed_color = ?,
                updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
        )
        .bind(name)
        .bind(channel_id)
        .bind(schedule_time)
        .bind(schedule_days)
        .bind(timezone)
        .bind(message_type)
        .bind(message_content)
        .bind(embed_title)
        .bind(embed_description)
        .bind(embed_color)
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn set_enabled(pool: &SqlitePool, id: i64, enabled: bool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE custom_reminders SET enabled = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(enabled)
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM custom_reminders WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomReminderPingRole {
    pub id: i64,
    pub reminder_id: i64,
    pub role_id: String,
}

impl CustomReminderPingRole {
    pub async fn get_by_reminder(
        pool: &SqlitePool,
        reminder_id: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, reminder_id, role_id FROM custom_reminder_ping_roles WHERE reminder_id = ?",
        )
        .bind(reminder_id)
        .fetch_all(pool)
        .await?;

        let mut roles = Vec::new();
        for row in rows {
            roles.push(Self {
                id: row.get("id"),
                reminder_id: row.get("reminder_id"),
                role_id: row.get("role_id"),
            });
        }

        Ok(roles)
    }

    pub async fn set_roles(
        pool: &SqlitePool,
        reminder_id: i64,
        role_ids: &[String],
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        sqlx::query("DELETE FROM custom_reminder_ping_roles WHERE reminder_id = ?")
            .bind(reminder_id)
            .execute(&mut *tx)
            .await?;

        for role_id in role_ids {
            sqlx::query(
                "INSERT INTO custom_reminder_ping_roles (reminder_id, role_id) VALUES (?, ?)",
            )
            .bind(reminder_id)
            .bind(role_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_by_reminder(
        pool: &SqlitePool,
        reminder_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM custom_reminder_ping_roles WHERE reminder_id = ?")
            .bind(reminder_id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomReminderSubscription {
    pub id: i64,
    pub user_id: String,
    pub reminder_id: i64,
    pub subscribed_at: DateTime<Utc>,
}

impl CustomReminderSubscription {
    pub async fn get_by_user(pool: &SqlitePool, user_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, user_id, reminder_id, subscribed_at
             FROM custom_reminder_subscriptions WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut subs = Vec::new();
        for row in rows {
            subs.push(Self {
                id: row.get("id"),
                user_id: row.get("user_id"),
                reminder_id: row.get("reminder_id"),
                subscribed_at: parse_sqlite_datetime(&row.get::<String, _>("subscribed_at")),
            });
        }

        Ok(subs)
    }

    pub async fn get_by_reminder(
        pool: &SqlitePool,
        reminder_id: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, user_id, reminder_id, subscribed_at
             FROM custom_reminder_subscriptions WHERE reminder_id = ?",
        )
        .bind(reminder_id)
        .fetch_all(pool)
        .await?;

        let mut subs = Vec::new();
        for row in rows {
            subs.push(Self {
                id: row.get("id"),
                user_id: row.get("user_id"),
                reminder_id: row.get("reminder_id"),
                subscribed_at: parse_sqlite_datetime(&row.get::<String, _>("subscribed_at")),
            });
        }

        Ok(subs)
    }

    pub async fn subscribe(
        pool: &SqlitePool,
        user_id: &str,
        reminder_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR IGNORE INTO custom_reminder_subscriptions (user_id, reminder_id) VALUES (?, ?)",
        )
        .bind(user_id)
        .bind(reminder_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn unsubscribe(
        pool: &SqlitePool,
        user_id: &str,
        reminder_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM custom_reminder_subscriptions WHERE user_id = ? AND reminder_id = ?",
        )
        .bind(user_id)
        .bind(reminder_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_by_id(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM custom_reminder_subscriptions WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomReminderLog {
    pub id: i64,
    pub reminder_id: i64,
    pub execution_time: DateTime<Utc>,
    pub status: String,
    pub error_message: Option<String>,
    pub channel_sent: bool,
    pub dm_count: i64,
    pub dm_failed_count: i64,
    pub created_at: DateTime<Utc>,
}

impl CustomReminderLog {
    pub async fn create(
        pool: &SqlitePool,
        reminder_id: i64,
        status: &str,
        error_message: Option<&str>,
        channel_sent: bool,
        dm_count: i64,
        dm_failed_count: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO custom_reminder_logs
                (reminder_id, execution_time, status, error_message, channel_sent, dm_count, dm_failed_count)
             VALUES (?, CURRENT_TIMESTAMP, ?, ?, ?, ?, ?)",
        )
        .bind(reminder_id)
        .bind(status)
        .bind(error_message)
        .bind(channel_sent)
        .bind(dm_count)
        .bind(dm_failed_count)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_recent_by_reminder(
        pool: &SqlitePool,
        reminder_id: i64,
        limit: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, reminder_id, execution_time, status, error_message,
                    channel_sent, dm_count, dm_failed_count, created_at
             FROM custom_reminder_logs WHERE reminder_id = ?
             ORDER BY created_at DESC LIMIT ?",
        )
        .bind(reminder_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        let mut logs = Vec::new();
        for row in rows {
            logs.push(Self {
                id: row.get("id"),
                reminder_id: row.get("reminder_id"),
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
