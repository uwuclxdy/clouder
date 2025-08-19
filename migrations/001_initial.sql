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