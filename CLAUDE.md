# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

**Build and Run:**
- `cargo run` - Build and run the bot
- `RUST_LOG=info cargo run` - Run with detailed logging
- `cargo build` - Build only
- `cargo check` - Fast compile check

**Testing:**
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run specific test
- `cargo test --package clouder --test mod` - Run integration tests

**Development:**
- `cargo clippy` - Lint checking
- `cargo fmt` - Code formatting

## Architecture Overview

**Core Framework:** Discord bot built with Serenity + Poise framework, with integrated Axum web server running in the same process.

**Key Components:**
- **Discord Bot:** Serenity + Poise for slash commands and event handling
- **Web Dashboard:** Axum server with Discord OAuth2 authentication
- **Database:** SQLite with sqlx for persistence
- **State Management:** Shared `AppState` containing config, database pool, cache, and HTTP client

**Project Structure:**
```
src/
├── main.rs              # Entry point, bot setup, web server startup
├── config.rs            # All configuration management (environment variables)
├── commands/            # Slash command implementations
├── database/            # Database models and operations
├── events/              # Discord event handlers (message, member events)
├── external/            # External API integrations (OpenAI, etc.)
├── utils/               # Shared utility functions
├── web/                 # Web dashboard (routes, templates, auth)
└── tests/               # All tests organized by module
```

**Database Design:**
- Uses SQLite with migration system in `migrations/` directory
- Auto-creates database and runs migrations on startup
- Key tables: `selfrole_configs`, `selfrole_roles`, `welcome_goodbye_configs`, `mediaonly_configs`

**Authentication:**
- Web dashboard requires Discord OAuth2 login
- Administrator permission required for server configuration
- Non-administrators get read-only access
- Session management with secure cookies

**Configuration Management:**
- All settings centralized in `src/config.rs`
- Environment variables loaded from `.env` file (see `.env.example`)
- Per-server settings stored in database
- Runtime config through `AppState` shared across components

**Key Patterns:**
- **Commands:** Each slash command in separate file under `src/commands/`
- **Database Operations:** Grouped by feature in `src/database/` with struct-based models
- **Event Handling:** Centralized in `src/events/event_handler()` with feature-specific modules
- **Web Routes:** RESTful API structure with HTML templates in `src/web/templates/`
- **Error Handling:** Graceful failures with user-friendly messages, extensive logging

**Integration Points:**
- Discord bot and web server share same `AppState` for database and Discord API access
- Background cleanup tasks run via Tokio spawned tasks
- All Discord interactions (commands, events) have access to web config for dashboard links

**Before Implementing New Features:**
- **Read Docs First:** Read `bot_requirements.md` and `checklist.md` before starting work
- **Check Related Code:** Read the files that you're working on to see if there are any similar or related features
