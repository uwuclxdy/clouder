-- 012: Drop the legacy plaintext `api_key` column from dashboard_users.
-- Migration 010 already wiped its contents to user_id placeholders and moved
-- credential lookup to api_key_hash. SQLite < 3.35 cannot drop columns in
-- place, so we recreate the table to avoid leaving a misleading column behind.

DROP TABLE IF EXISTS dashboard_users_new;

CREATE TABLE dashboard_users_new (
    user_id                TEXT    PRIMARY KEY NOT NULL,
    api_key_hash           TEXT,
    oauth_token            TEXT,
    oauth_token_updated_at INTEGER,
    username               TEXT,
    avatar                 TEXT,
    created_at             INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at             INTEGER NOT NULL DEFAULT (unixepoch())
);

INSERT INTO dashboard_users_new (
    user_id, api_key_hash, oauth_token, oauth_token_updated_at,
    username, avatar, created_at, updated_at
)
SELECT user_id, api_key_hash, oauth_token, oauth_token_updated_at,
       username, avatar, created_at, updated_at
FROM dashboard_users;

DROP TABLE dashboard_users;
ALTER TABLE dashboard_users_new RENAME TO dashboard_users;

CREATE UNIQUE INDEX IF NOT EXISTS idx_dashboard_users_api_key_hash
    ON dashboard_users(api_key_hash) WHERE api_key_hash IS NOT NULL;
