-- 005: UwUfy toggles

CREATE TABLE IF NOT EXISTS uwufy_toggles (
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    toggled_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(guild_id, user_id)
);

CREATE INDEX IF NOT EXISTS uwufy_toggles_guild_id ON uwufy_toggles(guild_id);
CREATE INDEX IF NOT EXISTS uwufy_toggles_user_id ON uwufy_toggles(user_id);
