# clouder-core

Shared core library consumed by the bot binary and the web crate. Contains configuration, database access, business logic orchestration, and utility functions.

## Module architecture

```
lib.rs
  |- config.rs              AppState, Config hierarchy, env loading
  |- crypto.rs              AES-256-GCM encrypt/decrypt, HMAC helpers for the dashboard
  |- database/
  |    |- mod.rs             initialize_database(), migration runner
  |    |- dashboard_sessions.rs  DashboardSession, session/CSRF management
  |    |- dashboard_users.rs     DashboardUser, API key hashing
  |    |- guild_cache.rs         CachedGuild, per-user guild list cache (TTL 1 h)
  |    |- guild_configs.rs       GuildConfig (timezone, command_prefix, embed_color)
  |    |- mediaonly.rs           MediaOnlyConfig
  |    |- reminders.rs           ReminderConfig, CustomReminder, subscriptions, user settings
  |    |- selfroles.rs           SelfRoleConfig, SelfRoleRole, SelfRoleCooldown, SelfRoleLabel
  |    |- uwufy.rs               UwufyToggle
  |    '- welcome_goodbye.rs     WelcomeGoodbyeConfig
  |- external/
  |    |- github.rs              GitHub API client
  |    |- github_trending.rs     GitHub trending scraper
  |    |- huggingface.rs         HuggingFace API client
  |    '- tinyfox.rs             TinyFox API client
  |- shared/
  |    |- mod.rs             business logic orchestrator
  |    '- models.rs          DTOs: ChannelInfo, RoleInfo, UserPermissions, GuildCacheEntry, etc.
  |- signal/                 (reserved, currently empty)
  '- utils/
       |- mod.rs             embed color, permissions, timestamps, duration, URL/time validation
       |- content_detection.rs  media type detection on messages
       '- welcome_goodbye.rs    embed builder, placeholder replacement
```

**Dependency direction**: `shared/mod.rs` sits at the top -- it calls into `database::*` and `utils::*`. The `database` modules use `utils::parse_sqlite_datetime`. The `utils` modules are leaf nodes with no internal dependencies.

## Feature flag

`llm` -- when enabled, adds `clouder-llm` as a dependency and places an `Option<clouder_llm::LlmClient>` field on `AppState`. Forwarded from the workspace root's `llm` feature.

## config

### Config

Root configuration loaded from environment variables via `Config::from_env()`. Struct hierarchy:

```
Config
  |- discord: DiscordConfig     (token, application_id, bot_owner)
  |- web: WebConfig             (api_base, bind_addr, oauth: OAuthConfig, embed: EmbedConfig,
  |                              session_secret, api_key_pepper, oauth_encryption_key)
  |- database: DatabaseConfig   (url)
  |- llm: LlmConfig             (provider, base_url, api_key, model, temperature, max_tokens,
  |                              timeout_seconds, system_prompt, stop, allowed_users, ...)
  |- github_token: Option<String>
  |- scheduler_interval: u64
  '- default_timezone: String
```

`Config::test_config()` returns a hardcoded fixture for unit tests (in-memory SQLite).

### AppState

```rust
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<SqlitePool>,
    pub http: Arc<Http>,
    #[cfg(feature = "llm")]
    pub llm_client: Option<clouder_llm::LlmClient>,
}
```

Constructed once at startup via `AppState::new(config, db, http)`. Shared across the entire application as the Poise `Data` type and Axum `State`.

## database

> **note:** create only the tables needed for the functionality that you're currently working on!

### initialize_database

`pub async fn initialize_database(db_url: &str) -> Result<SqlitePool>` -- creates the `data/` directory and SQLite file if missing, connects, and runs all migrations sequentially. Migrations are embedded at compile time via `include_str!`.

### selfroles

| Method | Signature |
|--------|-----------|
| `SelfRoleConfig::get_by_id` | `(pool, id: i64) -> Result<Option<Self>>` |
| `SelfRoleConfig::create` | `(pool, guild_id, channel_id, title, body, selection_type) -> Result<Self>` |
| `SelfRoleConfig::get_by_guild` | `(pool, guild_id: &str) -> Result<Vec<Self>>` |
| `SelfRoleConfig::get_by_message_id` | `(pool, message_id: &str) -> Result<Option<Self>>` |
| `SelfRoleConfig::update` | `(&mut self, pool, title, body, selection_type) -> Result<()>` |
| `SelfRoleConfig::update_message_id` | `(&mut self, pool, message_id: &str) -> Result<()>` |
| `SelfRoleConfig::update_channel_id` | `(&mut self, pool, channel_id: &str) -> Result<()>` |
| `SelfRoleConfig::delete` | `(&self, pool) -> Result<()>` |
| `SelfRoleConfig::delete_by_message_id` | `(pool, message_id: &str) -> Result<bool>` |
| `SelfRoleConfig::get_roles` | `(&self, pool) -> Result<Vec<SelfRoleRole>>` |
| `SelfRoleRole::create` | `(pool, config_id: i64, role_id, emoji) -> Result<Self>` |
| `SelfRoleRole::delete_by_config_id` | `(pool, config_id: i64) -> Result<()>` |
| `SelfRoleCooldown::create` | `(pool, user_id, role_id, guild_id, expires_at) -> Result<()>` -- INSERT OR REPLACE |
| `SelfRoleCooldown::check_cooldown` | `(pool, user_id, role_id, guild_id) -> Result<bool>` |
| `SelfRoleCooldown::cleanup_expired` | `(pool) -> Result<()>` -- deletes rows past expiry |
| `SelfRoleLabel::get` | `(pool, guild_id, role_id) -> Result<Option<Self>>` |
| `SelfRoleLabel::upsert` | `(pool, guild_id, role_id, name) -> Result<()>` |
| `SelfRoleLabel::upsert_many` | `(pool, guild_id, pairs: &[(&str, &str)]) -> Result<()>` |
| `SelfRoleLabel::get_all_for_guild` | `(pool, guild_id) -> Result<HashMap<String, String>>` |

### mediaonly

| Method | Signature |
|--------|-----------|
| `MediaOnlyConfig::get_by_channel` | `(pool, guild_id, channel_id) -> Result<Option<Self>>` |
| `MediaOnlyConfig::get_by_guild` | `(pool, guild_id) -> Result<Vec<Self>>` |
| `MediaOnlyConfig::upsert` | `(pool, guild_id, channel_id, enabled) -> Result<()>` |
| `MediaOnlyConfig::upsert_with_config` | `(pool, guild_id, channel_id, allow_links, allow_attachments, allow_gifs, allow_stickers) -> Result<()>` |
| `MediaOnlyConfig::toggle` | `(pool, guild_id, channel_id) -> Result<bool>` -- returns new state |
| `MediaOnlyConfig::delete` | `(pool, guild_id, channel_id) -> Result<()>` |

### welcome_goodbye

| Method | Signature |
|--------|-----------|
| `WelcomeGoodbyeConfig::get_config` | `(pool, guild_id) -> Result<Option<Self>>` |
| `WelcomeGoodbyeConfig::upsert_config` | `(pool, config: &Self) -> Result<()>` -- preserves original `created_at` |
| `get_member_placeholders` | `(user, guild_name, member_count, member) -> HashMap<String, String>` |

Config struct has separate fields for welcome and goodbye: `*_enabled`, `*_channel_id`, `*_message_type`, `*_message_content`, and embed fields (`*_title`, `*_description`, `*_color`, `*_footer`, `*_thumbnail`, `*_image`, `*_timestamp`).

### dashboard_sessions

| Method | Signature |
|--------|-----------|
| `DashboardSession::create` | `(db, user_id, ttl_seconds: i64) -> Result<Self>` |
| `DashboardSession::get_active` | `(db, session_id) -> Result<Option<Self>>` |
| `DashboardSession::delete` | `(db, session_id) -> Result<()>` |
| `DashboardSession::delete_expired` | `(db) -> Result<u64>` |
| `DashboardSession::csrf_matches` | `(&self, presented: &str) -> bool` -- constant-time compare |

### dashboard_users

| Item | Purpose |
|------|---------|
| `DashboardUser` | Stored dashboard user record |
| `hash_api_key(pepper, key)` | HMAC-SHA256 API key hash |

### guild_cache

| Item | Purpose |
|------|---------|
| `CachedGuild` | Cached guild entry (id, name, icon, permissions) |
| `GUILD_CACHE_TTL_SECONDS` | Cache TTL constant (3600 s) |

### guild_configs

| Item | Purpose |
|------|---------|
| `GuildConfig` | Per-guild config (timezone, command_prefix, embed_color) |
| `DEFAULT_TIMEZONE` | `"UTC"` |
| `DEFAULT_COMMAND_PREFIX` | `"!"` |

### reminders

Key types: `ReminderConfig`, `ReminderType`, `ReminderPingRole`, `ReminderSubscription`, `ReminderLog`, `CustomReminder`, `CustomReminderPingRole`, `CustomReminderSubscription`, `CustomReminderLog`, `UserSettings`, `GuildConfig`.

### uwufy

| Item | Purpose |
|------|---------|
| `UwufyToggle` | Per-user uwufy enable flag for a guild |

## shared

The orchestration layer. All functions take `&AppState` and coordinate between database operations, Discord API calls, and utility functions.

### Self-roles

| Function | What it does |
|----------|-------------|
| `list_selfroles(app_state, guild_id)` | Fetches configs + roles from DB, enriches with role labels |
| `create_selfrole(app_state, guild_id, user_id, payload)` | Validates, checks managed-role guard, creates DB record, deploys Discord message |
| `update_selfrole(app_state, guild_id, config_id, user_id, payload)` | Validates, edits Discord message in-place or redeploys, updates DB |
| `delete_selfrole(app_state, guild_id, config_id)` | Deletes Discord message + DB record |
| `format_selfrole_button_label(emoji, label)` | Prepends emoji to label string |

### Welcome/Goodbye

| Function | What it does |
|----------|-------------|
| `get_welcome_goodbye_config(app_state, guild_id)` | Returns config or defaults |
| `update_welcome_goodbye_config(app_state, guild_id, payload)` | Merges payload into existing config, validates URLs and lengths, upserts |
| `send_test_welcome_message(app_state, guild_id, msg_type, user_id)` | Builds and sends a test message to the configured channel |

### MediaOnly

| Function | What it does |
|----------|-------------|
| `list_mediaonly_configs(app_state, guild_id)` | Returns all configs for guild |
| `create_or_update_mediaonly_config(app_state, guild_id, channel_id, payload)` | Upserts with content type flags |
| `delete_mediaonly_config(app_state, guild_id, channel_id)` | Removes config |

### Guild config

| Function | What it does |
|----------|-------------|
| `get_guild_config(app_state, guild_id)` | Returns timezone, command_prefix, embed_color |
| `update_guild_config(app_state, guild_id, payload)` | Upserts guild config fields |
| `get_guild_about(app_state, guild_id)` | Returns guild info, channel/role counts, feature flags, bot config summary |

### Guild cache

| Function | What it does |
|----------|-------------|
| `refresh_guild_cache(state, user_id, access_token)` | Fetches user + bot guild lists, intersects by management permissions, updates DB cache, returns `(guilds, updated)` |

### Uwufy

| Function | What it does |
|----------|-------------|
| `list_uwufy_members(app_state, guild_id)` | Returns all non-bot guild members with their uwufy state |
| `toggle_uwufy_member(app_state, guild_id, user_id, enabled)` | Sets or toggles uwufy for a user |
| `disable_all_uwufy(app_state, guild_id)` | Disables uwufy for all members in a guild |

### Reminders

| Function | What it does |
|----------|-------------|
| `get_reminders_config(app_state, guild_id)` | Returns all reminder configs + custom reminders for a guild |
| `upsert_reminder_config(app_state, guild_id, payload)` | Creates or updates a built-in reminder config, validates timezone/times/lengths |
| `get_user_reminder_settings(app_state, user_id)` | Returns user timezone and DM-enabled flag |
| `update_user_reminder_settings(app_state, user_id, timezone, dm_enabled)` | Upserts user reminder settings |
| `list_user_subscriptions(app_state, user_id)` | Lists active reminder and custom-reminder subscriptions |
| `add_user_subscription(app_state, user_id, config_id)` | Subscribes user to a reminder config |
| `remove_user_subscription(app_state, user_id, config_id)` | Unsubscribes user from a reminder config |
| `remove_subscription_by_id(app_state, subscription_id)` | Deletes a subscription by its DB id |
| `get_custom_reminders(app_state, guild_id)` | Returns all custom reminders for a guild |
| `create_custom_reminder(app_state, guild_id, payload)` | Creates a custom reminder (max 10 per guild) |
| `update_custom_reminder(app_state, guild_id, reminder_id, payload)` | Updates a custom reminder |
| `delete_custom_reminder(app_state, guild_id, reminder_id)` | Deletes a custom reminder |

### Discord API helpers

| Function | What it does |
|----------|-------------|
| `get_guild_channels(app_state, guild_id)` | Fetches text + news channels via Discord HTTP |
| `get_guild_roles(app_state, guild_id)` | Fetches non-managed, non-@everyone roles |
| `send_dm_to_user(http, user_id, content)` | Opens a DM channel and sends content, splitting at 2000 chars |

### Interaction helpers

| Function | What it does |
|----------|-------------|
| `check_interaction_expired(error)` | Logs expired interactions (code 10062) at debug instead of error |

### shared::models

Simple DTOs used across the web API boundary:

- `ChannelInfo` -- id, name, channel_type, position
- `RoleInfo` -- id, name, color, position, mentionable
- `UserPermissions` -- permissions bitfield, is_admin, is_owner
- `CreateSelfRoleRequest` -- user_id, title, body, selection_type, channel_id, roles
- `SelfRoleData` -- role_id, emoji
- `GuildCacheEntry` -- id, name, icon, permissions

## utils

### Core utilities

| Function | Signature | Purpose |
|----------|-----------|---------|
| `get_embed_color` | `async (app_state, guild_id: Option<u64>) -> Color` | Reads per-guild config color, falls back to `config.web.embed.default_color` |
| `has_permission` | `(perms: Permissions, flag: Permissions) -> bool` | Bitfield permission check |
| `discord_timestamp` | `(timestamp: i64, style: char) -> String` | Formats `<t:TS:S>` Discord timestamp markup |
| `format_duration` | `(seconds: u64) -> String` | Human-readable `Xd Xh Xm Xs` |
| `format_count` | `(n: u64) -> String` | Locale-style number formatting |
| `parse_sqlite_datetime` | `(str) -> DateTime<Utc>` | Parses `%Y-%m-%d %H:%M:%S`, falls back to now |
| `parse_hhmm` | `(s: &str) -> Option<NaiveTime>` | Parses `HH:MM` time string |
| `is_valid_hhmm` | `(s: &str) -> bool` | Validates `HH:MM` format |
| `is_valid_https_url` | `(s: &str) -> bool` | Validates `https://` URL with a public host |

### utils::content_detection

Operates on `serenity::model::channel::Message`:

| Function | What it checks |
|----------|---------------|
| `has_link` | URL regex `https?://\S+` |
| `has_embedded_link` | Non-empty `message.embeds` |
| `has_attachment` | Non-empty `message.attachments` |
| `has_gif` | GIF file attachments, `.gif` URLs, Tenor/Giphy links, embed media with `.gif` |
| `has_sticker` | Non-empty `message.sticker_items` |
| `has_allowed_content` | Combines all checks against per-channel content type flags |

### utils::welcome_goodbye

| Item | Purpose |
|------|---------|
| `EmbedConfig<'a>` | Borrowed config struct for embed building |
| `build_embed(config, placeholders)` | Constructs `CreateEmbed` from config, applies placeholder replacement |
| `replace_placeholders(content, placeholders)` | Replaces `{key}` patterns in strings |

## crypto

| Function | Purpose |
|----------|---------|
| `encrypt(key_bytes: &[u8; 32], plaintext: &[u8]) -> Result<String>` | AES-256-GCM encrypt, returns hex blob |
| `decrypt(key_bytes: &[u8; 32], hex_blob: &str) -> Result<Vec<u8>>` | AES-256-GCM decrypt |
