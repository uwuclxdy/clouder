-- 007: Dashboard users
CREATE TABLE IF NOT EXISTS dashboard_users (
    user_id    TEXT    PRIMARY KEY NOT NULL,
    api_key    TEXT    NOT NULL UNIQUE,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX IF NOT EXISTS idx_dashboard_users_api_key ON dashboard_users(api_key);
