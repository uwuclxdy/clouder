# clouder

Discord bot built with Serenity + Poise, Axum web dashboard, SQLite database.

## Workspace

```
clouder/              main binary + lib (bot runtime, commands, events)
clouder-core/         shared core (config, database, business logic, utilities)
clouder-llm/          optional OpenAI-compatible LLM client (feature: "llm")
clouder-web/          Axum web dashboard (feature: "web")
```

Both `web` and `llm` features are on by default.

## Key paths

- `src/lib.rs` -- crate root, `run()` entry point, all public re-exports
- `src/commands/` -- poise slash commands (about, help, mediaonly, purge, selfroles)
- `src/events/` -- event handlers (mentions, mediaonly, member join/leave, selfrole buttons)
- `src/tests/` -- all tests, one file per module (about_tests, commands_tests, config_tests, database_tests, events_tests, help_tests, purge_tests, utils_tests, welcome_goodbye_tests)
- `clouder-core/src/config.rs` -- `Config`, `AppState` (the Poise `Data` type)
- `clouder-core/src/database/` -- SQLite models (selfroles, mediaonly, welcome_goodbye)
- `clouder-core/src/shared/mod.rs` -- business logic orchestrator (largest file in core)
- `clouder-core/src/utils/` -- embed color, permissions, timestamps, content detection
- `clouder-core/migrations/` -- SQL migrations (001-004), embedded at compile time
- `clouder-llm/src/openai.rs` -- `OpenAIClient` and request/response types
- `clouder-web/src/lib.rs` -- Axum server entry, `clouder_web::run(app_state)`
- `clouder-web/src/api.rs` -- REST handlers delegating to `clouder_core::shared::*`

## Build and check

Run `../cargo.sh` after changes -- it runs fmt, fix, clippy fix, then clippy (all in release mode). Fix any warnings it reports.

## Rules

- never document code that is self-explanatory
- never run `cargo run`
- use `clean-code` skill when writing code
- prioritize solving problems by removing, simplifying, or reusing existing code
- find and reuse existing functions/helpers to avoid duplication
- cover new features or changes with tests in `src/tests/` (one file per module)
- all user-facing text: short, concise, lowercase (except abbreviations like API, OS)
- logging: use `tracing` macros (`trace!`, `debug!`, `info!`, `warn!`, `error!`)
- embed colors: `utils::get_default_embed_color(app_state)` in commands, `get_default_embed_color(&state)` in web handlers
- `AppState` flows everywhere as the Poise `Data` type and Axum `State`
- database: `data/db.sqlite` SQLite with WAL mode

## More

for more info, see the README files in each crate and the code itself. The code is meant to be clean and self-explanatory, so reading it is the best way to understand how it works.

# clouder (crate)

Discord bot binary and library crate. Bootstraps the Serenity/Poise runtime, registers slash commands, hooks event handlers, and optionally starts the web dashboard.

## Workspace

```
clouder           (this crate)  Discord bot binary + lib
clouder-core      shared core   Config, database, business logic, utilities
clouder-llm       optional      OpenAI-compatible LLM client
clouder-web       optional      Axum web dashboard
```

### Dependency graph

```
clouder
  |- clouder-core          (always)
  |- clouder-llm           (feature = "llm")
  '- clouder-web           (feature = "web")

clouder-core
  '- clouder-llm           (feature = "llm", forwarded from root)

clouder-llm                (standalone -- reqwest, serde, anyhow)
clouder-web                (depends on clouder-core at the Rust layer)
```

### Feature flags

| Flag  | Default | Effect                                                                                     |
| ----- | ------- | ------------------------------------------------------------------------------------------ |
| `web` | on      | Enables `clouder-web` -- starts Axum web dashboard alongside the bot                       |
| `llm` | on      | Enables `clouder-llm` -- activates `clouder-core/llm`, places `OpenAIClient` in `AppState` |

## Entry point

`main.rs` calls `clouder::run()`, which spawns a tokio multi-thread runtime and runs `async_main()`. The startup sequence:

1. Create `.env` from `.env.example` if missing (exits with instructions)
2. Load `Config::from_env()` and initialize logging
3. `initialize_database()` -- creates SQLite file + runs migrations
4. Build `AppState` with `Config`, `SqlitePool`, Discord `Http`, and optionally `OpenAIClient`
5. Register Poise commands and the event handler
6. Start background cooldown cleanup (every 5 minutes)
7. Start the Serenity client and `clouder_web::run(app_state)` concurrently via `try_join!` (if `web`)

## Type aliases

```rust
type Data = AppState;   // Poise framework data type
type Error = Box<dyn std::error::Error + Send + Sync>;
```

`AppState` flows through every command and event handler as the Poise `Data` type, giving access to config, database pool, Discord HTTP client, and the optional LLM client.

## Module structure

```
src/
  lib.rs                 crate root, public re-exports, run() + async_main()
  main.rs                thin binary entry point
  logging.rs             tracing-subscriber init, re-exports debug/info/warn/error macros
  commands/
    mod.rs               declares command modules
    about.rs             /about {bot,server,user,role,channel} -- BOT_START_TIME static
    help.rs              /help [category] -- CommandInfo, CommandCategory, get_all_commands()
    mediaonly.rs          /mediaonly [channel] [enabled]
    purge.rs             /purge [number | message_id]
    selfroles.rs         /selfroles -- links to web dashboard
  events/
    mod.rs               event_handler dispatch -- Ready, InteractionCreate, Message, member events
    bot_mentioned.rs     LLM response on @mention, ai_retry button handler
    mediaonly_handler.rs content detection + auto-delete for media-only channels
    member_events.rs     welcome/goodbye on join/leave, Database/AppStateKey TypeMapKeys
    selfroles.rs         button interaction handler, message cleanup on delete
  tests/
    mod.rs               create_test_db(), create_test_app_state() helpers
    ...                  per-module test files
```

## Public API

Re-exported from `clouder-core`:

```rust
pub use clouder_core::config::{AppState, Config};
pub use clouder_core::database::selfroles::{SelfRoleConfig, SelfRoleCooldown, SelfRoleRole};
pub use clouder_core::database::welcome_goodbye::WelcomeGoodbyeConfig;
pub use clouder_core::shared::models::{ChannelInfo, CreateSelfRoleRequest, RoleInfo, SelfRoleData, UserPermissions};
pub use clouder_core::shared::{create_selfrole, delete_selfrole, get_guild_channels, get_guild_roles, list_selfroles};
```

Own exports:

```rust
pub use commands::{about, help, mediaonly, purge, selfroles};  // Poise command functions
pub use events::event_handler;
pub use logging::{debug, error, info};
pub fn run() -> Result<()>  // starts the runtime
```

## Background tasks

- **Cooldown cleanup**: `SelfRoleCooldown::cleanup_expired(&db)` runs every 5 minutes via `start_cleanup_task()`, removing expired self-role cooldown entries from the database.
