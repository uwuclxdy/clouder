# clouder-core

Shared core library consumed by the bot binary and the web crate. Contains configuration, database access, business logic orchestration, and utility functions.

## Module architecture

```
lib.rs
  |- config.rs              AppState, Config hierarchy, env loading
  |- database/
  |    |- mod.rs             initialize_database(), migration runner
  |    |- selfroles.rs       SelfRoleConfig, SelfRoleRole, SelfRoleCooldown
  |    |- mediaonly.rs       MediaOnlyConfig
  |    '- welcome_goodbye.rs WelcomeGoodbyeConfig, placeholder helpers
  |- shared/
  |    |- mod.rs             business logic orchestrator (selfroles, welcome/goodbye, mediaonly)
  |    '- models.rs          DTOs: ChannelInfo, RoleInfo, UserPermissions, etc.
  '- utils/
       |- mod.rs             embed color, permissions, timestamps, duration
       |- content_detection.rs  media type detection on messages
       '- welcome_goodbye.rs    embed builder, placeholder replacement
```

**Dependency direction**: `shared/mod.rs` sits at the top -- it calls into `database::*` and `utils::*`. The `database` modules use `utils::parse_sqlite_datetime`. The `utils` modules are leaf nodes with no internal dependencies.

## Feature flag

`llm` -- when enabled, adds `clouder-llm` as a dependency and places an `Option<OpenAIClient>` field on `AppState`. Forwarded from the workspace root's `llm` feature.

## config

### Config

Root configuration loaded from environment variables via `Config::from_env()`. Struct hierarchy:

```
Config
  |- discord: DiscordConfig     (token, application_id, bot_owner)
  |- web: WebConfig             (host, port, base_url, oauth: OAuthConfig, embed: EmbedConfig)
  |- database: DatabaseConfig   (url)
  '- openai: OpenAIConfig       (enabled, base_url, api_key, model, temperature, max_tokens, ...)
```

`Config::test_config()` returns a hardcoded fixture for unit tests (in-memory SQLite).

### AppState

```rust
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<SqlitePool>,
    pub http: Arc<Http>,
    pub openai_client: Option<OpenAIClient>,  // only when "llm" feature active
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
| `SelfRoleConfig::get_by_id` | `(pool, id: i64) -> Option<Self>` |
| `SelfRoleConfig::create` | `(pool, guild_id, channel_id, title, body, selection_type) -> Self` |
| `SelfRoleConfig::get_by_guild` | `(pool, guild_id: &str) -> Vec<Self>` |
| `SelfRoleConfig::get_by_message_id` | `(pool, message_id: &str) -> Option<Self>` |
| `SelfRoleConfig::update` | `(&mut self, pool, title, body, selection_type)` |
| `SelfRoleConfig::update_message_id` | `(&mut self, pool, message_id)` |
| `SelfRoleConfig::update_channel_id` | `(&mut self, pool, channel_id)` |
| `SelfRoleConfig::delete` | `(&self, pool)` |
| `SelfRoleConfig::delete_by_message_id` | `(pool, message_id) -> bool` |
| `SelfRoleConfig::get_roles` | `(&self, pool) -> Vec<SelfRoleRole>` |
| `SelfRoleRole::create` | `(pool, config_id, role_id, emoji) -> Self` |
| `SelfRoleRole::delete_by_config_id` | `(pool, config_id)` |
| `SelfRoleCooldown::create` | `(pool, user_id, role_id, guild_id, expires_at)` -- INSERT OR REPLACE |
| `SelfRoleCooldown::check_cooldown` | `(pool, user_id, role_id, guild_id) -> bool` |
| `SelfRoleCooldown::cleanup_expired` | `(pool)` -- deletes rows past expiry |

### mediaonly

| Method | Signature |
|--------|-----------|
| `MediaOnlyConfig::get_by_channel` | `(pool, guild_id, channel_id) -> Option<Self>` |
| `MediaOnlyConfig::get_by_guild` | `(pool, guild_id) -> Vec<Self>` |
| `MediaOnlyConfig::upsert` | `(pool, guild_id, channel_id, enabled)` |
| `MediaOnlyConfig::upsert_with_config` | `(pool, guild_id, channel_id, allow_links, allow_attachments, allow_gifs, allow_stickers)` |
| `MediaOnlyConfig::toggle` | `(pool, guild_id, channel_id) -> bool` -- returns new state |
| `MediaOnlyConfig::delete` | `(pool, guild_id, channel_id)` |

### welcome_goodbye

| Method | Signature |
|--------|-----------|
| `WelcomeGoodbyeConfig::get_config` | `(pool, guild_id) -> Option<Self>` |
| `WelcomeGoodbyeConfig::upsert_config` | `(pool, config: &Self)` -- preserves original `created_at` |
| `get_member_placeholders` | `(user, guild_name, member_count, member) -> HashMap<String, String>` |

Config struct has separate fields for welcome and goodbye: `*_enabled`, `*_channel_id`, `*_message_type`, `*_message_content`, and embed fields (`*_title`, `*_description`, `*_color`, `*_footer`, `*_thumbnail`, `*_image`, `*_timestamp`).

## shared

The orchestration layer. All functions take `&AppState` and coordinate between database operations, Discord API calls, and utility functions.

### Self-roles

| Function | What it does |
|----------|-------------|
| `list_selfroles(app_state, guild_id)` | Fetches configs + roles from DB, enriches with channel info |
| `create_selfrole(app_state, guild_id, user_id, payload)` | Validates, checks role hierarchy, creates DB record, deploys Discord message |
| `update_selfrole(app_state, guild_id, config_id, user_id, payload)` | Validates, edits Discord message in-place or redeploys, updates DB |
| `delete_selfrole(app_state, guild_id, config_id)` | Deletes Discord message + DB record |
| `format_selfrole_button_label(emoji, label)` | Prepends emoji to label string |

### Welcome/Goodbye

| Function | What it does |
|----------|-------------|
| `get_welcome_goodbye_config(app_state, guild_id)` | Returns config or defaults |
| `update_welcome_goodbye_config(app_state, guild_id, payload)` | Merges payload into existing config, upserts |
| `send_test_welcome_message(app_state, guild_id, msg_type, user_id)` | Builds and sends a test message to the configured channel |

### MediaOnly

| Function | What it does |
|----------|-------------|
| `list_mediaonly_configs(app_state, guild_id)` | Returns all configs for guild |
| `create_or_update_mediaonly_config(app_state, guild_id, channel_id, payload)` | Upserts with content type flags |
| `delete_mediaonly_config(app_state, guild_id, channel_id)` | Removes config |

### Discord API helpers

| Function | What it does |
|----------|-------------|
| `get_guild_channels(app_state, guild_id)` | Fetches text + news channels via Discord HTTP |
| `get_guild_roles(app_state, guild_id)` | Fetches non-@everyone roles |

### shared::models

Simple DTOs used across the web API boundary:

- `ChannelInfo` -- id, name, channel_type, position
- `RoleInfo` -- id, name, color, position, mentionable
- `UserPermissions` -- permissions bitfield, is_admin, is_owner
- `CreateSelfRoleRequest` -- user_id, title, body, selection_type, channel_id, roles
- `SelfRoleData` -- role_id, emoji

## utils

### Core utilities

| Function | Signature | Purpose |
|----------|-----------|---------|
| `get_default_embed_color` | `(app_state) -> Color` | Reads `config.web.embed.default_color` |
| `get_bot_channel_permissions` | `(http, guild_id, channel_id) -> Option<BotChannelPermissions>` | Computes effective bot permissions in a channel |
| `bot_has_permission_in_channel` | `(http, guild_id, channel_id, check_fn) -> bool` | Wrapper; returns true for DMs |
| `can_bot_manage_role` | `(bot_role_positions, target_position) -> bool` | Role hierarchy check |
| `get_bot_role_positions` | `(bot_member, guild_roles) -> Vec<u16>` | Maps member roles to position values |
| `discord_timestamp` | `(timestamp, style) -> String` | Formats `<t:TS:S>` Discord timestamp markup |
| `format_duration` | `(seconds) -> String` | Human-readable `Xd Xh Xm Xs` |
| `parse_sqlite_datetime` | `(str) -> DateTime<Utc>` | Parses `%Y-%m-%d %H:%M:%S`, falls back to now |

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
