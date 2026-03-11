# Database Schema & Models

## selfrole system

### `selfrole_configs`
- primary key `id` (int)
- `guild_id` (text), `channel_id` (text), `message_id` (text), `title` (text), `body` (text), `selection_type` (text: 'radio' or 'multiple'), `created_at` (timestamp), `updated_at` (timestamp)

### `selfrole_roles`
- primary key `id` (int)
- `config_id` (int) *fk → selfrole_configs(id)*, `role_id` (text), `emoji` (text)

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
- `command_prefix` (text), `embed_color` (integer, nullable), `timezone` (text), `created_at` (datetime), `updated_at` (datetime)

### `reminder_configs`
- primary key `id` (int)
- `guild_id` (text), `reminder_type` (text: 'wysi'/'custom'), `enabled` (boolean), `channel_id` (text, nullable), `message_type` (text: 'embed'/'text'), `message_content` (text, nullable), `embed_title` (text, nullable), `embed_description` (text, nullable), `embed_color` (integer, nullable), `wysi_morning_time` (text), `wysi_evening_time` (text), `timezone` (text), `created_at` (datetime), `updated_at` (datetime)

### `reminder_ping_roles`
- primary key `id` (int)
- `config_id` (int) *fk → reminder_configs(id)*, `role_id` (text)

### `reminder_subscriptions`
- primary key `id` (int)
- `user_id` (text), `config_id` (int) *fk → reminder_configs(id)*, `subscribed_at` (datetime)

### `reminder_logs`
- primary key `id` (int)
- `config_id` (int) *fk → reminder_configs(id)*, `execution_time` (datetime), `status` (text: 'success'/'error'/'partial'), `error_message` (text, nullable), `channel_sent` (boolean), `dm_count` (int), `dm_failed_count` (int), `created_at` (datetime)

## welcome / goodbye

### `welcome_goodbye_configs`
- primary key `guild_id` (text)
- `welcome_enabled` (boolean), `goodbye_enabled` (boolean), `welcome_channel_id` (text, nullable), `goodbye_channel_id` (text, nullable), `welcome_message_type` (text: 'embed'/'text'), `goodbye_message_type` (text: 'embed'/'text'), `welcome_message_content` (text, nullable), `goodbye_message_content` (text, nullable), plus various embed fields (title, description, color, footer, thumbnail, image, timestamp booleans) for both welcome and goodbye, and timestamps

## mediaonly feature

### `mediaonly_configs`
- primary key `id` (int)
- `guild_id` (text), `channel_id` (text), `enabled` (boolean), `allow_links` (boolean), `allow_attachments` (boolean), `allow_gifs` (boolean), `allow_stickers` (boolean), `created_at` (datetime), `updated_at` (datetime)

## other tables

### `user_guild_cache`
- composite key `(user_id, guild_id)`
- `user_id` (text), `guild_id` (text), `name` (text), `icon` (text, nullable), `updated_at` (int unixepoch)

### `uwufy_toggles`
- composite key `(guild_id, user_id)`
- `guild_id` (text), `user_id` (text), `enabled` (boolean), `toggled_at` (datetime)

### `dashboard_users`
- primary key `user_id` (text)
- `api_key` (text unique), `created_at` (int unixepoch), `updated_at` (int unixepoch)
