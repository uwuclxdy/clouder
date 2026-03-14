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
