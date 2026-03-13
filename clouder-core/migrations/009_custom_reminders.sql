-- 009: Custom reminders

CREATE TABLE IF NOT EXISTS custom_reminders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id TEXT NOT NULL,
    name TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    channel_id TEXT,
    schedule_time TEXT NOT NULL DEFAULT '12:00',
    schedule_days TEXT NOT NULL DEFAULT '',
    timezone TEXT NOT NULL DEFAULT 'UTC',
    message_type TEXT NOT NULL DEFAULT 'embed' CHECK (message_type IN ('embed', 'text')),
    message_content TEXT,
    embed_title TEXT,
    embed_description TEXT,
    embed_color INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (guild_id) REFERENCES guild_configs (guild_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS custom_reminder_ping_roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    reminder_id INTEGER NOT NULL,
    role_id TEXT NOT NULL,
    FOREIGN KEY (reminder_id) REFERENCES custom_reminders (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS custom_reminder_subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    reminder_id INTEGER NOT NULL,
    subscribed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (user_id, reminder_id),
    FOREIGN KEY (user_id) REFERENCES user_settings (user_id) ON DELETE CASCADE,
    FOREIGN KEY (reminder_id) REFERENCES custom_reminders (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS custom_reminder_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    reminder_id INTEGER NOT NULL,
    execution_time DATETIME NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('success', 'error', 'partial')),
    error_message TEXT,
    channel_sent BOOLEAN DEFAULT FALSE,
    dm_count INTEGER DEFAULT 0,
    dm_failed_count INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (reminder_id) REFERENCES custom_reminders (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS custom_reminders_guild_id ON custom_reminders (guild_id);
CREATE INDEX IF NOT EXISTS custom_reminders_enabled ON custom_reminders (enabled);
CREATE INDEX IF NOT EXISTS custom_reminders_guild_enabled ON custom_reminders (guild_id, enabled);
CREATE INDEX IF NOT EXISTS custom_reminder_ping_roles_rid ON custom_reminder_ping_roles (reminder_id);
CREATE INDEX IF NOT EXISTS custom_reminder_subs_user ON custom_reminder_subscriptions (user_id);
CREATE INDEX IF NOT EXISTS custom_reminder_subs_rid ON custom_reminder_subscriptions (reminder_id);
CREATE INDEX IF NOT EXISTS custom_reminder_logs_rid ON custom_reminder_logs (reminder_id);
CREATE INDEX IF NOT EXISTS custom_reminder_logs_created ON custom_reminder_logs (created_at);
