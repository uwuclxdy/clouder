# Database

clouder stores everything in a single SQLite database.

- **File:** `data/db.sqlite`
- **Connection:** `DATABASE_URL` (code default `data/db.sqlite`; `.env.example` ships `sqlite:data/db.sqlite`)
- **Pragmas:** `foreign_keys = ON`. (No WAL pragma is set, so SQLite uses its default rollback journal.)
- **Migrations:** 13 SQL files (`001`–`013`) embedded at compile time via `include_str!` and applied by a
  custom runner in `database/mod.rs`. The runner tracks applied versions in a `schema_migrations` ledger,
  splits statements safely, and recovers partially-applied upgrades. `initialize_database()` creates the
  `data/` directory and the file if missing, then runs pending migrations.

## Schema

### Self-roles

**`selfrole_configs`** · key `id`
`guild_id`, `channel_id`, `message_id`, `title`, `body`, `selection_type` (`radio` or `multiple`),
`created_at`, `updated_at`.

**`selfrole_roles`** · key `id`
`config_id` (fk → `selfrole_configs`), `role_id`, `emoji`.

**`selfrole_cooldowns`** · key `(user_id, role_id, guild_id)`
`expires_at`.

**`selfrole_labels`** · key `(guild_id, role_id)`
`name`, `updated_at`.

### Reminders and configuration

**`user_settings`** · key `user_id`
`timezone`, `dm_reminders_enabled`, `created_at`, `updated_at`.

**`guild_configs`** · key `guild_id`
`command_prefix`, `embed_color` (nullable), `timezone`, `created_at`, `updated_at`.

**`reminder_configs`** · key `id`
`guild_id`, `reminder_type` (`wysi`/`custom`), `enabled`, `channel_id`, `message_type` (`embed`/`text`),
`message_content`, embed fields, `wysi_morning_time`, `wysi_evening_time`, `timezone`, timestamps.

**`reminder_ping_roles`** · key `id`
`config_id` (fk → `reminder_configs`), `role_id`.

**`reminder_subscriptions`** · key `id`
`user_id`, `config_id` (fk → `reminder_configs`), `subscribed_at`.

**`reminder_logs`** · key `id`
`config_id` (fk → `reminder_configs`), `execution_time`, `status` (`success`/`error`/`partial`),
`error_message`, `channel_sent`, `dm_count`, `dm_failed_count`, `created_at`.

### Custom reminders (migration 009)

**`custom_reminders`** · key `id`
`guild_id` (fk → `guild_configs`), `name`, `enabled`, `channel_id`, `schedule_time`, `schedule_days`,
`timezone`, `message_type` (`embed`/`text`), `message_content`, `embed_title`, `embed_description`,
`embed_color`.

**`custom_reminder_ping_roles`** · key `id`
`reminder_id` (fk → `custom_reminders`), `role_id`.

**`custom_reminder_subscriptions`** · key `id`
`user_id` (fk → `user_settings`), `reminder_id` (fk → `custom_reminders`), unique `(user_id, reminder_id)`.

**`custom_reminder_logs`** · key `id`
`reminder_id` (fk → `custom_reminders`), `status` (`success`/`error`/`partial`), `error_message`,
`channel_sent`, `dm_count`, `dm_failed_count`.

### Welcome / goodbye

**`welcome_goodbye_configs`** · key `guild_id`
`welcome_enabled`, `goodbye_enabled`, channels, message types (`embed`/`text`), message content, and embed
fields (title, description, color, footer, thumbnail, image, timestamp) for both welcome and goodbye, plus timestamps.

### Media-only

**`mediaonly_configs`** · key `id`
`guild_id`, `channel_id`, `enabled`, `allow_links`, `allow_attachments`, `allow_gifs`, `allow_stickers`,
`created_at`, `updated_at`. Unique on `(guild_id, channel_id)`.

### Dashboard and caches

**`dashboard_users`** · key `user_id`
`api_key_hash` (HMAC-SHA256 with `API_KEY_PEPPER`, for auth lookup),
`api_key_ciphertext` (AES-256-GCM with `OAUTH_ENCRYPTION_KEY`, so the user can view their own key),
`oauth_token` (AES-256-GCM ciphertext), `oauth_token_updated_at`, `username`, `avatar`, timestamps.

**`dashboard_sessions`** · key `session_id`
`user_id`, `csrf_token`, `expires_at`, `created_at`. Swept periodically by the web server and the bot's cleanup task.

**`user_guild_cache`** · key `(user_id, guild_id)`
`name`, `icon`, `permissions`, `updated_at` (unix epoch), `expires_at` (TTL added in migration 011 to force
a permission refresh).

**`uwufy_toggles`** · key `(guild_id, user_id)`
`enabled`, `toggled_at`.

> [!NOTE]
> The encryption and hashing keys for `dashboard_users` come from the secrets on the
> [Configuration](Configuration#required) page. Rotating `OAUTH_ENCRYPTION_KEY` makes stored tokens
> unreadable; rotating `API_KEY_PEPPER` invalidates all API keys.
