-- Migration: 003_welcome_goodbye.sql
-- Welcome and Goodbye messages system

CREATE TABLE IF NOT EXISTS welcome_goodbye_configs (
    guild_id TEXT PRIMARY KEY,
    welcome_enabled BOOLEAN DEFAULT FALSE,
    goodbye_enabled BOOLEAN DEFAULT FALSE,
    welcome_channel_id TEXT,
    goodbye_channel_id TEXT,
    welcome_message_type TEXT DEFAULT 'embed' CHECK (welcome_message_type IN ('embed', 'text')),
    goodbye_message_type TEXT DEFAULT 'embed' CHECK (goodbye_message_type IN ('embed', 'text')),
    welcome_message_content TEXT,
    goodbye_message_content TEXT,
    -- Welcome embed fields
    welcome_embed_title TEXT,
    welcome_embed_description TEXT,
    welcome_embed_color INTEGER,
    welcome_embed_footer TEXT,
    welcome_embed_thumbnail TEXT,
    welcome_embed_image TEXT,
    welcome_embed_timestamp BOOLEAN DEFAULT FALSE,
    -- Goodbye embed fields
    goodbye_embed_title TEXT,
    goodbye_embed_description TEXT,
    goodbye_embed_color INTEGER,
    goodbye_embed_footer TEXT,
    goodbye_embed_thumbnail TEXT,
    goodbye_embed_image TEXT,
    goodbye_embed_timestamp BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_welcome_goodbye_configs_guild ON welcome_goodbye_configs(guild_id);
