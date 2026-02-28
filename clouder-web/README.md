# clouder-web

Axum-based web server for the clouder dashboard. Gated behind the workspace `web` feature flag.

## Structure

```
clouder-web/
  src/
    lib.rs    Axum server setup, route registration
    api.rs    REST handler implementations (thin wrappers around clouder-core::shared)
```

## Entry point

```rust
pub async fn run(app_state: AppState) -> Result<()>
```

Starts an Axum HTTP server on the address derived from `config.web.api_base`. Receives the same `AppState` used by the bot, giving it access to the database pool and Discord HTTP client.

## REST endpoints

All handlers delegate to `clouder_core::shared::*` functions.

| Method | Path                                                  | Handler                                     |
| ------ | ----------------------------------------------------- | ------------------------------------------- |
| GET    | `/api/guild/{guild_id}/channels`                      | `shared::get_guild_channels`                |
| GET    | `/api/guild/{guild_id}/roles`                         | `shared::get_guild_roles`                   |
| GET    | `/api/selfroles/{guild_id}`                           | `shared::list_selfroles`                    |
| POST   | `/api/selfroles/{guild_id}`                           | `shared::create_selfrole`                   |
| DELETE | `/api/selfroles/{guild_id}/{config_id}`               | `shared::delete_selfrole`                   |
| GET    | `/api/welcome-goodbye/{guild_id}/config`              | `shared::get_welcome_goodbye_config`        |
| POST   | `/api/welcome-goodbye/{guild_id}/config`              | `shared::update_welcome_goodbye_config`     |
| POST   | `/api/welcome-goodbye/{guild_id}/test/{message_type}` | `shared::send_test_welcome_message`         |
| GET    | `/api/mediaonly/{guild_id}`                           | `shared::list_mediaonly_configs`            |
| POST   | `/api/mediaonly/{guild_id}`                           | `shared::create_or_update_mediaonly_config` |
| PUT    | `/api/mediaonly/{guild_id}/{channel_id}`              | `shared::create_or_update_mediaonly_config` |
| DELETE | `/api/mediaonly/{guild_id}/{channel_id}`              | `shared::delete_mediaonly_config`           |
