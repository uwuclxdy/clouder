-- Migration 004: Media-only channels configuration
CREATE TABLE IF NOT EXISTS mediaonly_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,

    -- What content types are allowed (if false, will be deleted)
    allow_links BOOLEAN NOT NULL DEFAULT TRUE,
    allow_attachments BOOLEAN NOT NULL DEFAULT TRUE,
    allow_gifs BOOLEAN NOT NULL DEFAULT TRUE,
    allow_stickers BOOLEAN NOT NULL DEFAULT TRUE,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(guild_id, channel_id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_mediaonly_guild ON mediaonly_configs(guild_id);
CREATE INDEX IF NOT EXISTS idx_mediaonly_channel ON mediaonly_configs(channel_id);
CREATE INDEX IF NOT EXISTS idx_mediaonly_enabled ON mediaonly_configs(enabled);
