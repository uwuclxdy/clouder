-- Migration 002: Reminders System and User Settings

-- User settings table for timezone and preferences
CREATE TABLE IF NOT EXISTS user_settings (
                                             user_id TEXT PRIMARY KEY,
                                             timezone TEXT DEFAULT 'UTC',
                                             dm_reminders_enabled BOOLEAN DEFAULT TRUE,
                                             created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                                             updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Server reminder configurations
CREATE TABLE IF NOT EXISTS reminder_configs (
                                                id INTEGER PRIMARY KEY AUTOINCREMENT,
                                                guild_id TEXT NOT NULL,
                                                reminder_type TEXT NOT NULL CHECK(reminder_type IN ('wysi', 'femboy_friday', 'custom')),
    enabled BOOLEAN DEFAULT TRUE,
    channel_id TEXT NOT NULL,

    -- Message configuration (either embed or text)
    message_type TEXT NOT NULL DEFAULT 'embed' CHECK(message_type IN ('embed', 'text')),

    -- For text messages
    message_content TEXT,

    -- For embed messages (JSON)
    embed_title TEXT,
    embed_description TEXT,
    embed_color INTEGER DEFAULT 5793266, -- Default purple color
    embed_footer TEXT,
    embed_image_url TEXT,
    embed_thumbnail_url TEXT,
    embed_fields TEXT, -- JSON array of fields

-- WYSI specific settings
    wysi_morning_time TEXT DEFAULT '07:27', -- HH:MM format for AM
    wysi_evening_time TEXT DEFAULT '19:27', -- HH:MM format for PM

-- Femboy Friday specific settings
    ff_trigger_time TEXT DEFAULT '00:00', -- Time when Friday starts
    ff_gif_url TEXT, -- Current GIF URL (will be randomized via Giphy API later)

-- Custom reminder settings (for future implementation)
    custom_name TEXT,
    custom_schedule TEXT, -- Cron expression or similar
    custom_next_trigger DATETIME,

    -- Timezone for this reminder config
    timezone TEXT DEFAULT 'UTC',

    -- Last triggered timestamps
    last_triggered_at DATETIME,
    next_trigger_at DATETIME,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(guild_id, reminder_type, custom_name)
    );

-- Roles to ping for reminders
CREATE TABLE IF NOT EXISTS reminder_ping_roles (
                                                   id INTEGER PRIMARY KEY AUTOINCREMENT,
                                                   config_id INTEGER NOT NULL,
                                                   role_id TEXT NOT NULL,
                                                   FOREIGN KEY (config_id) REFERENCES reminder_configs(id) ON DELETE CASCADE,
    UNIQUE(config_id, role_id)
    );

-- User subscriptions to reminders (for DMs)
CREATE TABLE IF NOT EXISTS reminder_subscriptions (
                                                      id INTEGER PRIMARY KEY AUTOINCREMENT,
                                                      user_id TEXT NOT NULL,
                                                      config_id INTEGER NOT NULL,
                                                      subscribed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                                                      PRIMARY KEY (user_id, config_id),
    FOREIGN KEY (config_id) REFERENCES reminder_configs(id) ON DELETE CASCADE
    );

-- Reminder execution log (for debugging and history)
CREATE TABLE IF NOT EXISTS reminder_logs (
                                             id INTEGER PRIMARY KEY AUTOINCREMENT,
                                             config_id INTEGER NOT NULL,
                                             execution_time DATETIME NOT NULL,
                                             status TEXT NOT NULL CHECK(status IN ('success', 'failed', 'skipped')),
    error_message TEXT,
    users_notified INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (config_id) REFERENCES reminder_configs(id) ON DELETE CASCADE
    );

-- Guild configuration updates (general settings)
CREATE TABLE IF NOT EXISTS guild_configs (
                                             guild_id TEXT PRIMARY KEY,
                                             command_prefix TEXT DEFAULT '!',
                                             embed_color INTEGER DEFAULT 5793266, -- Default purple
                                             created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                                             updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_reminder_configs_guild ON reminder_configs(guild_id);
CREATE INDEX IF NOT EXISTS idx_reminder_configs_enabled ON reminder_configs(enabled);
CREATE INDEX IF NOT EXISTS idx_reminder_configs_next_trigger ON reminder_configs(next_trigger_at);
CREATE INDEX IF NOT EXISTS idx_reminder_subscriptions_user ON reminder_subscriptions(user_id);
CREATE INDEX IF NOT EXISTS idx_reminder_subscriptions_config ON reminder_subscriptions(config_id);
CREATE INDEX IF NOT EXISTS idx_reminder_logs_config ON reminder_logs(config_id);
CREATE INDEX IF NOT EXISTS idx_reminder_logs_execution ON reminder_logs(execution_time);
CREATE INDEX IF NOT EXISTS idx_reminder_ping_roles_config ON reminder_ping_roles(config_id);
CREATE INDEX IF NOT EXISTS idx_reminder_ping_roles_role ON reminder_ping_roles(role_id);
CREATE INDEX IF NOT EXISTS idx_guild_configs_guild ON guild_configs(guild_id);
CREATE INDEX IF NOT EXISTS idx_user_settings_user ON user_settings(user_id);
CREATE INDEX IF NOT EXISTS idx_user_settings_timezone ON user_settings(timezone);
CREATE INDEX IF NOT EXISTS idx_user_settings_dm_reminders ON user_settings(dm_reminders_enabled);
CREATE INDEX IF NOT EXISTS idx_user_settings_created_at ON user_settings(created_at);
