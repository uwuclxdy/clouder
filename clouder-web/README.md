# clouder-web

Axum-based web server for the clouder dashboard. Gated behind the workspace `web` feature flag. It serves
server-rendered dashboard pages, the Discord OAuth2 flow, static assets, and a JSON API. All API handlers
are thin wrappers over `clouder_core::shared::*`.

## Structure

```
clouder-web/
  src/
    lib.rs        Axum server setup, route registration, security middleware, cookie-key derivation
    api.rs        JSON API handlers (delegate to clouder_core::shared)
    auth.rs       Discord OAuth2 login / callback / logout
    dashboard.rs  server-rendered HTML page handlers
    session.rs    session + CSRF helpers
```

## Entry point

```rust
pub async fn run(app_state: AppState) -> Result<()>
```

Binds an Axum HTTP server to `config.web.bind_addr` (`WEB_BIND_ADDR`); `config.web.api_base` (`API_BASE`)
is used for OAuth redirects, not for binding. Receives the same `AppState` as the bot, so it shares the
database pool and Discord HTTP client.

## Security

- Discord OAuth2 login; sessions in signed cookies (`SignedCookieJar`) whose key is HKDF-SHA256-derived from `SESSION_SECRET`.
- Server-side sessions (`dashboard_sessions`) carrying a CSRF token, echoed via the `X-CSRF-Token` header.
- OAuth tokens encrypted at rest (AES-256-GCM, `OAUTH_ENCRYPTION_KEY`); API keys hashed (HMAC-SHA256, `API_KEY_PEPPER`).
- Per-IP rate limiting (`tower_governor`): 100 req/s burst 300; the DM endpoint is capped at 1 req/s burst 5.
- Security headers (CSP, `X-Frame-Options`, `Referrer-Policy`, `Permissions-Policy`, `X-Content-Type-Options`) and a 256 KB body limit.
- A background task sweeps expired `dashboard_sessions` every 15 minutes.

## Routes

Pages: `/`, `/login`, `/servers`, `/profile`, `/dashboard/{guild_id}` (redirect), and
`/dashboard/{guild_id}/{selfroles|welcome-goodbye|about|mediaonly|uwufy|reminders}`.
Auth: `/auth/{login,callback,logout}`. Static: `/static/style.css`, `/static/app.js`.

### JSON API (`/api/*`)

| Method | Path | Delegates to |
| ------ | ---- | ------------ |
| POST | `/api/guilds/refresh` | `refresh_guild_cache` |
| GET | `/api/guild/{guild_id}/channels` | `get_guild_channels` |
| GET | `/api/guild/{guild_id}/roles` | `get_guild_roles` |
| GET | `/api/guild/{guild_id}/about` | `get_guild_about` |
| GET / POST | `/api/guild/{guild_id}/config` | `get_guild_config` / `update_guild_config` |
| GET / POST | `/api/selfroles/{guild_id}` | `list_selfroles` / `create_selfrole` |
| PUT / DELETE | `/api/selfroles/{guild_id}/{config_id}` | `update_selfrole` / `delete_selfrole` |
| GET / POST | `/api/welcome-goodbye/{guild_id}/config` | `get_welcome_goodbye_config` / `update_welcome_goodbye_config` |
| POST | `/api/welcome-goodbye/{guild_id}/test/{message_type}` | `send_test_welcome_message` |
| GET / POST | `/api/mediaonly/{guild_id}` | `list_mediaonly_configs` / `create_or_update_mediaonly_config` |
| PUT / DELETE | `/api/mediaonly/{guild_id}/{channel_id}` | `create_or_update_mediaonly_config` / `delete_mediaonly_config` |
| GET / DELETE | `/api/uwufy/{guild_id}` | `list_uwufy_members` / `disable_all_uwufy` |
| PUT | `/api/uwufy/{guild_id}/{user_id}` | `toggle_uwufy_member` |
| GET / POST | `/api/reminders/{guild_id}` | `get_reminders_config` / `upsert_reminder_config` |
| POST | `/api/reminders/{guild_id}/{config_id}/test` | reminder test send |
| GET / POST | `/api/custom-reminders/{guild_id}` | `get_custom_reminders` / `create_custom_reminder` |
| PUT / DELETE | `/api/custom-reminders/{guild_id}/{reminder_id}` | `update_custom_reminder` / `delete_custom_reminder` |
| POST | `/api/custom-reminders/{guild_id}/{reminder_id}/test` | custom reminder test send |
| GET / POST | `/api/user/dm_reminders` | `get_user_reminder_settings` / `update_user_reminder_settings` |
| GET | `/api/user/subscriptions` | `list_user_subscriptions` |
| POST | `/api/user/subscribe/{config_id}` | `add_user_subscription` |
| DELETE | `/api/user/unsubscribe/{config_id}` | `remove_user_subscription` |
| DELETE | `/api/user/subscription/{id}` | `remove_subscription_by_id` |
| POST | `/api/profile/regenerate-key` | regenerate the caller's dashboard API key |
| POST | `/api/{user_id}` | send a DM (separate rate-limited sub-router) |
