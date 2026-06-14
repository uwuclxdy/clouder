# Database Schema & Models

## selfrole system

### `selfrole_configs`
- primary key `id` (int)
- `guild_id` (text), `channel_id` (text), `message_id` (text unique nullable), `title` (text), `body` (text), `selection_type` (text: 'radio' or 'multiple'), `created_at` (datetime), `updated_at` (datetime)

### `selfrole_roles`
- primary key `id` (int)
- `config_id` (int) *fk -> selfrole_configs(id)*, `role_id` (text), `emoji` (text)

### `selfrole_cooldowns`
- primary key `(user_id, role_id, guild_id)`
- `user_id` (text), `role_id` (text), `guild_id` (text), `expires_at` (datetime)

### `selfrole_labels`
- primary key `(guild_id, role_id)`
- `guild_id` (text), `role_id` (text), `name` (text), `updated_at` (datetime)

## reminders & configuration

### `user_settings`
- primary key `user_id` (text)
- `timezone` (text), `dm_reminders_enabled` (boolean), `created_at` (datetime), `updated_at` (datetime)

### `guild_configs`
- primary key `guild_id` (text)
- `command_prefix` (text), `embed_color` (text hex `#RRGGBB`, nullable; legacy integer values converted by migration 002), `timezone` (text), `created_at` (datetime), `updated_at` (datetime)

### `reminder_configs`
- primary key `id` (int)
- `guild_id` (text) *fk -> guild_configs(guild_id)*, `reminder_type` (text: 'wysi' or 'custom'), `enabled` (boolean), `channel_id` (text, nullable), `message_type` (text: 'embed' or 'text'), `message_content` (text, nullable), `embed_title` (text, nullable), `embed_description` (text, nullable), `embed_color` (integer, nullable), `wysi_morning_time` (text), `wysi_evening_time` (text), `timezone` (text), `created_at` (datetime), `updated_at` (datetime)
- unique constraint: `(guild_id, reminder_type)`

### `reminder_ping_roles`
- primary key `id` (int)
- `config_id` (int) *fk -> reminder_configs(id)*, `role_id` (text)

### `reminder_subscriptions`
- primary key `id` (int)
- `user_id` (text) *fk -> user_settings(user_id)*, `config_id` (int) *fk -> reminder_configs(id)*, `subscribed_at` (datetime)
- unique constraint: `(user_id, config_id)`

### `reminder_logs`
- primary key `id` (int)
- `config_id` (int) *fk -> reminder_configs(id)*, `execution_time` (datetime), `status` (text: 'success', 'error', or 'partial'), `error_message` (text, nullable), `channel_sent` (boolean), `dm_count` (int), `dm_failed_count` (int), `created_at` (datetime)

## custom reminders

### `custom_reminders`
- primary key `id` (int)
- `guild_id` (text) *fk -> guild_configs(guild_id)*, `name` (text), `enabled` (boolean), `channel_id` (text, nullable), `schedule_time` (text), `schedule_days` (text), `timezone` (text), `message_type` (text: 'embed' or 'text'), `message_content` (text, nullable), `embed_title` (text, nullable), `embed_description` (text, nullable), `embed_color` (integer, nullable), `created_at` (datetime), `updated_at` (datetime)

### `custom_reminder_ping_roles`
- primary key `id` (int)
- `reminder_id` (int) *fk -> custom_reminders(id)*, `role_id` (text)

### `custom_reminder_subscriptions`
- primary key `id` (int)
- `user_id` (text) *fk -> user_settings(user_id)*, `reminder_id` (int) *fk -> custom_reminders(id)*, `subscribed_at` (datetime)
- unique constraint: `(user_id, reminder_id)`

### `custom_reminder_logs`
- primary key `id` (int)
- `reminder_id` (int) *fk -> custom_reminders(id)*, `execution_time` (datetime), `status` (text: 'success', 'error', or 'partial'), `error_message` (text, nullable), `channel_sent` (boolean), `dm_count` (int), `dm_failed_count` (int), `created_at` (datetime)

## welcome / goodbye

### `welcome_goodbye_configs`
- primary key `guild_id` (text)
- `welcome_enabled` (boolean), `goodbye_enabled` (boolean), `welcome_channel_id` (text, nullable), `goodbye_channel_id` (text, nullable), `welcome_message_type` (text: 'embed' or 'text'), `goodbye_message_type` (text: 'embed' or 'text'), `welcome_message_content` (text, nullable), `goodbye_message_content` (text, nullable), plus embed fields for both welcome and goodbye: `*_embed_title`, `*_embed_description`, `*_embed_color` (integer), `*_embed_footer`, `*_embed_thumbnail`, `*_embed_image`, `*_embed_timestamp` (boolean), and `created_at` (datetime), `updated_at` (datetime)

## mediaonly feature

### `mediaonly_configs`
- primary key `id` (int)
- `guild_id` (text), `channel_id` (text), `enabled` (boolean), `allow_links` (boolean), `allow_attachments` (boolean), `allow_gifs` (boolean), `allow_stickers` (boolean), `created_at` (datetime), `updated_at` (datetime)
- unique constraint: `(guild_id, channel_id)`

## other tables

### `user_guild_cache`
- composite key `(user_id, guild_id)`
- `user_id` (text), `guild_id` (text), `name` (text), `icon` (text, nullable), `updated_at` (int unixepoch), `permissions` (int), `expires_at` (int unixepoch, TTL for cache invalidation)

### `uwufy_toggles`
- composite key `(guild_id, user_id)`
- `guild_id` (text), `user_id` (text), `enabled` (boolean), `toggled_at` (datetime)

### `dashboard_users`
- primary key `user_id` (text)
- `api_key_hash` (text unique nullable, HMAC-SHA256 hex with API_KEY_PEPPER, used for auth lookup)
- `api_key_ciphertext` (text nullable, AES-256-GCM(OAUTH_ENCRYPTION_KEY) hex, decrypted only on the user's own profile so they can view the key without regenerating)
- `oauth_token` (text nullable, AES-256-GCM ciphertext), `oauth_token_updated_at` (int unixepoch nullable)
- `username` (text nullable), `avatar` (text nullable)
- `created_at` (int unixepoch), `updated_at` (int unixepoch)

### `dashboard_sessions`
- primary key `session_id` (text)
- `user_id` (text), `csrf_token` (text), `expires_at` (int unixepoch), `created_at` (int unixepoch)

### `schema_migrations`
- internal migration ledger, created in code before any SQL files run
- primary key `version` (int), `name` (text), `applied_at` (int unixepoch)
