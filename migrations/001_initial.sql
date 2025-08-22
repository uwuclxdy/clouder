-- Initial schema for self-roles
CREATE TABLE selfrole_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    message_id TEXT UNIQUE,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    selection_type TEXT NOT NULL CHECK(selection_type IN ('radio', 'multiple')),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE selfrole_roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_id INTEGER NOT NULL,
    role_id TEXT NOT NULL,
    emoji TEXT NOT NULL,
    FOREIGN KEY (config_id) REFERENCES selfrole_configs(id) ON DELETE CASCADE
);

CREATE TABLE selfrole_cooldowns (
    user_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    guild_id TEXT NOT NULL,
    expires_at DATETIME NOT NULL,
    PRIMARY KEY (user_id, role_id, guild_id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS selfrole_roles_config_id ON selfrole_roles(config_id);
CREATE INDEX IF NOT EXISTS selfrole_cooldowns_expires_at ON selfrole_cooldowns(expires_at);
CREATE INDEX IF NOT EXISTS selfrole_cooldowns_user_id ON selfrole_cooldowns(user_id);
CREATE INDEX IF NOT EXISTS selfrole_cooldowns_role_id ON selfrole_cooldowns(role_id);
CREATE INDEX IF NOT EXISTS selfrole_cooldowns_guild_id ON selfrole_cooldowns(guild_id);
CREATE INDEX IF NOT EXISTS selfrole_configs_guild_id ON selfrole_configs(guild_id);
CREATE INDEX IF NOT EXISTS selfrole_configs_channel_id ON selfrole_configs(channel_id);
CREATE INDEX IF NOT EXISTS selfrole_configs_message_id ON selfrole_configs(message_id);
CREATE INDEX IF NOT EXISTS selfrole_configs_selection_type ON selfrole_configs(selection_type);
CREATE INDEX IF NOT EXISTS selfrole_roles_role_id ON selfrole_roles(role_id);
CREATE INDEX IF NOT EXISTS selfrole_roles_emoji ON selfrole_roles(emoji);
CREATE INDEX IF NOT EXISTS selfrole_configs_created_at ON selfrole_configs(created_at);
CREATE INDEX IF NOT EXISTS selfrole_configs_updated_at ON selfrole_configs(updated_at);
CREATE INDEX IF NOT EXISTS selfrole_configs_title ON selfrole_configs(title);

CREATE INDEX IF NOT EXISTS selfrole_roles_config_emoji ON selfrole_roles(config_id, emoji);
CREATE INDEX IF NOT EXISTS selfrole_cooldowns_user_guild ON selfrole_cooldowns(user_id, guild_id);
