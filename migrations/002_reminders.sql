-- Reminders system and user settings tables

-- User settings for timezone and DM preferences
CREATE TABLE IF NOT EXISTS user_settings (
    user_id TEXT PRIMARY KEY,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    dm_reminders_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Guild configurations for command prefix and embed colors
CREATE TABLE IF NOT EXISTS guild_configs (
    guild_id TEXT PRIMARY KEY,
    command_prefix TEXT NOT NULL DEFAULT '!',
    embed_color INTEGER DEFAULT NULL,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Reminder configurations for different types (WYSI, Femboy Friday, custom)
CREATE TABLE IF NOT EXISTS reminder_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id TEXT NOT NULL,
    reminder_type TEXT NOT NULL CHECK(reminder_type IN ('wysi', 'femboy_friday', 'custom')),
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    channel_id TEXT,
    message_type TEXT NOT NULL DEFAULT 'embed' CHECK(message_type IN ('embed', 'text')),
    message_content TEXT,
    embed_title TEXT,
    embed_description TEXT,
    embed_color INTEGER,
    wysi_morning_time TEXT DEFAULT '07:27',
    wysi_evening_time TEXT DEFAULT '19:27',
    femboy_friday_time TEXT DEFAULT '00:00',
    timezone TEXT NOT NULL DEFAULT 'UTC',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (guild_id) REFERENCES guild_configs(guild_id) ON DELETE CASCADE
);

-- Roles to ping for specific reminders
CREATE TABLE IF NOT EXISTS reminder_ping_roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_id INTEGER NOT NULL,
    role_id TEXT NOT NULL,
    FOREIGN KEY (config_id) REFERENCES reminder_configs(id) ON DELETE CASCADE
);

-- User subscriptions to reminders for DM notifications
CREATE TABLE IF NOT EXISTS reminder_subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    config_id INTEGER NOT NULL,
    subscribed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, config_id),
    FOREIGN KEY (user_id) REFERENCES user_settings(user_id) ON DELETE CASCADE,
    FOREIGN KEY (config_id) REFERENCES reminder_configs(id) ON DELETE CASCADE
);

-- Execution logs for reminders (tracking and debugging)
CREATE TABLE IF NOT EXISTS reminder_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_id INTEGER NOT NULL,
    execution_time DATETIME NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('success', 'error', 'partial')),
    error_message TEXT,
    channel_sent BOOLEAN DEFAULT FALSE,
    dm_count INTEGER DEFAULT 0,
    dm_failed_count INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (config_id) REFERENCES reminder_configs(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS user_settings_timezone ON user_settings(timezone);
CREATE INDEX IF NOT EXISTS user_settings_dm_enabled ON user_settings(dm_reminders_enabled);

CREATE INDEX IF NOT EXISTS guild_configs_timezone ON guild_configs(timezone);
CREATE INDEX IF NOT EXISTS guild_configs_prefix ON guild_configs(command_prefix);

CREATE INDEX IF NOT EXISTS reminder_configs_guild_id ON reminder_configs(guild_id);
CREATE INDEX IF NOT EXISTS reminder_configs_type ON reminder_configs(reminder_type);
CREATE INDEX IF NOT EXISTS reminder_configs_enabled ON reminder_configs(enabled);
CREATE INDEX IF NOT EXISTS reminder_configs_channel_id ON reminder_configs(channel_id);
CREATE INDEX IF NOT EXISTS reminder_configs_timezone ON reminder_configs(timezone);

CREATE INDEX IF NOT EXISTS reminder_ping_roles_config_id ON reminder_ping_roles(config_id);
CREATE INDEX IF NOT EXISTS reminder_ping_roles_role_id ON reminder_ping_roles(role_id);

CREATE INDEX IF NOT EXISTS reminder_subscriptions_user_id ON reminder_subscriptions(user_id);
CREATE INDEX IF NOT EXISTS reminder_subscriptions_config_id ON reminder_subscriptions(config_id);
CREATE INDEX IF NOT EXISTS reminder_subscriptions_subscribed_at ON reminder_subscriptions(subscribed_at);

CREATE INDEX IF NOT EXISTS reminder_logs_config_id ON reminder_logs(config_id);
CREATE INDEX IF NOT EXISTS reminder_logs_execution_time ON reminder_logs(execution_time);
CREATE INDEX IF NOT EXISTS reminder_logs_status ON reminder_logs(status);
CREATE INDEX IF NOT EXISTS reminder_logs_created_at ON reminder_logs(created_at);

-- Composite indexes for common queries
CREATE INDEX IF NOT EXISTS reminder_configs_guild_enabled ON reminder_configs(guild_id, enabled);
CREATE INDEX IF NOT EXISTS reminder_configs_type_enabled ON reminder_configs(reminder_type, enabled);
CREATE INDEX IF NOT EXISTS reminder_subscriptions_user_config ON reminder_subscriptions(user_id, config_id);
