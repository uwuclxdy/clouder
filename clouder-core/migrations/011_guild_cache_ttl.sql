-- 011: Add expires_at TTL to user_guild_cache to prevent stale permission
-- caches from granting access after the user is kicked/banned or permissions
-- are revoked on Discord's side. Default 0 forces a refresh on first read so
-- pre-existing rows can't bypass the new check.

ALTER TABLE user_guild_cache ADD COLUMN expires_at INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_guild_cache_expires_at ON user_guild_cache (expires_at);
