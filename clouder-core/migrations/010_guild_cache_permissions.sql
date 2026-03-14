-- 010: add permissions bitfield to guild cache
ALTER TABLE user_guild_cache ADD COLUMN permissions INTEGER NOT NULL DEFAULT 0;
