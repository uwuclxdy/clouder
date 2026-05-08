-- 010: Hash API keys, add OAuth token storage, add session revocation, add CSRF tokens
-- Existing api_key column is replaced by api_key_hash (HMAC-SHA256 hex of the
-- key, computed with the server-side API_KEY_PEPPER). Plaintext keys are
-- discarded on first request after deploy and users must regenerate from /profile.

ALTER TABLE dashboard_users ADD COLUMN api_key_hash TEXT;
ALTER TABLE dashboard_users ADD COLUMN oauth_token TEXT;
ALTER TABLE dashboard_users ADD COLUMN oauth_token_updated_at INTEGER;
ALTER TABLE dashboard_users ADD COLUMN username TEXT;
ALTER TABLE dashboard_users ADD COLUMN avatar TEXT;

DROP INDEX IF EXISTS idx_dashboard_users_api_key;
CREATE UNIQUE INDEX IF NOT EXISTS idx_dashboard_users_api_key_hash
    ON dashboard_users(api_key_hash) WHERE api_key_hash IS NOT NULL;

-- Force regeneration: existing plaintext keys can no longer authenticate.
-- Reuse user_id as the placeholder so the column-level UNIQUE constraint
-- (carried over from migration 007) doesn't reject the bulk update.
UPDATE dashboard_users SET api_key = user_id;

CREATE TABLE IF NOT EXISTS dashboard_sessions (
    session_id TEXT    PRIMARY KEY NOT NULL,
    user_id    TEXT    NOT NULL,
    csrf_token TEXT    NOT NULL,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX IF NOT EXISTS idx_dashboard_sessions_user_id
    ON dashboard_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_dashboard_sessions_expires_at
    ON dashboard_sessions(expires_at);
