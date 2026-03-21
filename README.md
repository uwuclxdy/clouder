# Clouder

A modular Discord bot built with Rust. Features slash commands, a web dashboard, LLM integration, scheduled reminders, and self-role management.

## Workspace

| Crate | Description |
|-------|-------------|
| `clouder` | Bot binary — runtime, commands, events, scheduler |
| `clouder-core` | Shared library — config, database, business logic, utilities |
| `clouder-llm` | OpenAI-compatible LLM client (optional, feature: `llm`) |
| `clouder-web` | Axum web dashboard with REST API (optional, feature: `web`) |

Both `web` and `llm` features are enabled by default.

## Commands

| Command | Description | Permission |
|---------|-------------|------------|
| `/about bot\|server\|user\|role\|channel` | Info and stats (uptime, RAM, CPU, latency) | — |
| `/help [category]` | List commands by category | — |
| `/selfroles` | Link to web dashboard for self-role setup | — |
| `/purge <count\|message_id>` | Bulk delete messages | Manage Messages |
| `/mediaonly <channel> [enabled]` | Toggle media-only mode on a channel | Manage Channels |
| `/channel delete\|clone\|nuke` | Channel management | Manage Channels |
| `/reminders` | View active reminders | — |
| `/hf latest\|trending` | Browse HuggingFace models | — |
| `/github <user> [repo]` | GitHub user or repo stats | — |
| `/gh-trending [period]` | Trending GitHub repos | — |
| `/uwufy [user]` | Toggle uwuify on a user | — |
| `/random` | Random number generator | — |

## Features

- **LLM Mentions** — Responds to @mentions and replies using a configurable LLM provider (whitelist-based, per-user cooldown)
- **Media-Only** — Auto-deletes non-media messages in configured channels
- **UwUify** — Replaces messages from uwuified users
- **Welcome/Goodbye** — Sends configurable messages on member join/leave with placeholders (`{user}`, `{server}`, `{member_count}`, etc.)
- **Self-Role Buttons** — Handles button interactions for role assignment with cooldowns
- **Message Cleanup** — Removes orphaned self-role data when messages are deleted

## Web Dashboard

An Axum REST API running alongside the bot. Provides endpoints for managing self-roles, welcome/goodbye configs, and media-only channels. Authenticated via Discord OAuth2 with signed session cookies.

## Setup

### Prerequisites

- Rust (edition 2024)
- SQLite

### 1. Clone and Configure

```sh
git clone https://github.com/uwuclxdy/clouder.git
cd clouder
cp .env.example .env
```

Edit `.env` and fill in the required values:

```sh
DISCORD_TOKEN=           # Bot token from Discord Developer Portal
DISCORD_CLIENT_ID=       # Application/client ID
DISCORD_CLIENT_SECRET=   # Client secret (used for OAuth2)
BOT_OWNER=               # Your Discord user ID
```

### 2. Optional Configuration

```sh
# Web Dashboard
API_BASE=https://your-domain.com    # Public URL for OAuth redirects
WEB_BIND_ADDR=127.0.0.1:8080       # Local bind address

# Embed Styling
EMBED_DEFAULT_COLOR=#FFFFFF         # Hex or decimal

# LLM Integration (set provider to enable)
LLM_PROVIDER=openai                 # openai or ollama
LLM_BASE_URL=                       # API endpoint
LLM_API_KEY=                        # Bearer token
LLM_MODEL=gpt-4o
LLM_ALLOWED_USERS=                  # Comma-separated user IDs

# GitHub API (optional, raises rate limit from 60 to 5000/hr)
GITHUB_TOKEN=

# Scheduler
SCHEDULER_INTERVAL=60               # Reminder check interval in seconds
DEFAULT_TIMEZONE=UTC                 # IANA timezone name
```

### 3. Build and Run

```sh
cargo build --release
./target/release/clouder
```

On first run the bot creates `data/db.sqlite`, runs all migrations, and registers slash commands globally. If `.env` is missing it generates one from `.env.example` and exits with instructions.

### Compile Without Optional Features

```sh
cargo build --release --no-default-features              # Bot only
cargo build --release --no-default-features --features web   # Bot + Web
cargo build --release --no-default-features --features llm   # Bot + LLM
```

## Bot Permissions

The bot needs these Discord permissions:

- Send Messages, Embed Links, Read Message History
- Manage Messages (purge, media-only)
- Manage Roles (self-role assignment)
- Manage Channels (channel commands, media-only)

Gateway intents: `GUILD_MESSAGES`, `GUILDS`, `MESSAGE_CONTENT`, `GUILD_MEMBERS`

## Database

SQLite with WAL mode at `data/db.sqlite`. Migrations run automatically on startup. Tables cover self-roles (configs, roles, cooldowns, labels), reminders (configs, subscriptions, logs), welcome/goodbye, media-only, uwuify toggles, dashboard users, and guild configs.

## Background Tasks

- **Cooldown Cleanup** — Purges expired self-role cooldowns every 5 minutes
- **Reminder Scheduler** — Checks and sends due reminders at a configurable interval (default 60s), with deduplication
