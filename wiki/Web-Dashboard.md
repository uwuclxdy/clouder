# Web Dashboard

An Axum HTTP server (`clouder-web`) that runs alongside the bot and shares its `AppState`, so it reaches
the same database pool and Discord HTTP client. Gated behind the `web` feature (on by default). It serves
server-rendered dashboard pages, the OAuth2 flow, static assets, and a JSON API.

## Entry point

```rust
pub async fn run(app_state: AppState) -> Result<()>
```

The bot starts the dashboard and the Serenity client concurrently via `try_join!`. The server binds to
`WEB_BIND_ADDR`; `API_BASE` is used for OAuth redirects.

## Authentication and security

- **Discord OAuth2** login. Routes: `GET /auth/login`, `GET /auth/callback`, `GET /auth/logout`.
- **Signed session cookies** via `axum-extra` `SignedCookieJar`. The signing key is derived from
  `SESSION_SECRET` with HKDF-SHA256, not used directly.
- **Server-side sessions** in the `dashboard_sessions` table, carrying a **CSRF token**. Authenticated
  pages embed it in a `<meta name="csrf-token">` tag and the JS sends it back as `X-CSRF-Token`.
- **OAuth tokens encrypted at rest** with `OAUTH_ENCRYPTION_KEY` (AES-256-GCM).
- **Dashboard API keys** hashed with `API_KEY_PEPPER` (HMAC-SHA256) for lookup, and stored AES-256-GCM
  encrypted so a user can view their own key from `/profile` without regenerating.
- **Per-IP rate limiting** (`tower_governor`): 100 req/s, burst 300 for the dashboard; the DM endpoint is
  capped at 1 req/s, burst 5.
- **Security headers** on every response: CSP, `X-Frame-Options: DENY`, `Referrer-Policy`,
  `Permissions-Policy`, `X-Content-Type-Options: nosniff`.
- **256 KB request body limit**.

See [Configuration](Configuration#required) for the secrets these depend on.

## Pages and assets

- Pages (server-rendered): `/`, `/login`, `/servers`, `/profile`, and
  `/dashboard/{guild_id}/{selfroles|welcome-goodbye|about|mediaonly|uwufy|reminders}`
  (plus `/dashboard/{guild_id}` which redirects).
- Static assets: `/static/style.css`, `/static/app.js`.

## JSON API

All handlers delegate to `clouder_core::shared::*`.

### Guild

| Method | Path | Delegates to |
|--------|------|--------------|
| POST | `/api/guilds/refresh` | `refresh_guild_cache` |
| GET | `/api/guild/{guild_id}/channels` | `get_guild_channels` |
| GET | `/api/guild/{guild_id}/roles` | `get_guild_roles` |
| GET | `/api/guild/{guild_id}/about` | `get_guild_about` |
| GET / POST | `/api/guild/{guild_id}/config` | `get_guild_config` / `update_guild_config` |

### Self-roles

| Method | Path | Delegates to |
|--------|------|--------------|
| GET | `/api/selfroles/{guild_id}` | `list_selfroles` |
| POST | `/api/selfroles/{guild_id}` | `create_selfrole` |
| PUT | `/api/selfroles/{guild_id}/{config_id}` | `update_selfrole` |
| DELETE | `/api/selfroles/{guild_id}/{config_id}` | `delete_selfrole` |

### Welcome / goodbye

| Method | Path | Delegates to |
|--------|------|--------------|
| GET / POST | `/api/welcome-goodbye/{guild_id}/config` | `get_welcome_goodbye_config` / `update_welcome_goodbye_config` |
| POST | `/api/welcome-goodbye/{guild_id}/test/{message_type}` | `send_test_welcome_message` |

### Media-only

| Method | Path | Delegates to |
|--------|------|--------------|
| GET / POST | `/api/mediaonly/{guild_id}` | `list_mediaonly_configs` / `create_or_update_mediaonly_config` |
| PUT / DELETE | `/api/mediaonly/{guild_id}/{channel_id}` | `create_or_update_mediaonly_config` / `delete_mediaonly_config` |

### UwUfy

| Method | Path | Delegates to |
|--------|------|--------------|
| GET | `/api/uwufy/{guild_id}` | `list_uwufy_members` |
| DELETE | `/api/uwufy/{guild_id}` | `disable_all_uwufy` |
| PUT | `/api/uwufy/{guild_id}/{user_id}` | `toggle_uwufy_member` |

### Reminders

| Method | Path | Delegates to |
|--------|------|--------------|
| GET / POST | `/api/reminders/{guild_id}` | `get_reminders_config` / `upsert_reminder_config` |
| POST | `/api/reminders/{guild_id}/{config_id}/test` | reminder test send |
| GET / POST | `/api/custom-reminders/{guild_id}` | `get_custom_reminders` / `create_custom_reminder` |
| PUT / DELETE | `/api/custom-reminders/{guild_id}/{reminder_id}` | `update_custom_reminder` / `delete_custom_reminder` |
| POST | `/api/custom-reminders/{guild_id}/{reminder_id}/test` | custom reminder test send |

### User

| Method | Path | Delegates to |
|--------|------|--------------|
| GET / POST | `/api/user/dm_reminders` | `get_user_reminder_settings` / `update_user_reminder_settings` |
| GET | `/api/user/subscriptions` | `list_user_subscriptions` |
| POST | `/api/user/subscribe/{config_id}` | `add_user_subscription` |
| DELETE | `/api/user/unsubscribe/{config_id}` | `remove_user_subscription` |
| DELETE | `/api/user/subscription/{id}` | `remove_subscription_by_id` |

### Profile and DM

| Method | Path | Notes |
|--------|------|-------|
| POST | `/api/profile/regenerate-key` | Regenerates the caller's dashboard API key |
| POST | `/api/{user_id}` | Sends a DM. Separate sub-router with the strict 1 req/s rate limit |

## Background task

The dashboard sweeps expired rows from `dashboard_sessions` every 15 minutes. (The bot's own cleanup task
also clears expired sessions every 5 minutes; see [Architecture](Architecture#background-tasks).)
