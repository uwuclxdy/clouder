# Installation

How to build and run clouder, choose feature flags, and grant the right Discord permissions.

## Prerequisites

- **Rust** (edition 2024)

SQLite ships bundled through `sqlx`, so there is nothing else to install.

## 1. Clone and configure

```sh
git clone https://github.com/uwuclxdy/clouder.git
cd clouder
cp .env.example .env
```

Fill the required Discord values in `.env`:

```sh
DISCORD_TOKEN=           # Bot token from the Discord Developer Portal
DISCORD_CLIENT_ID=       # Application / client ID
DISCORD_CLIENT_SECRET=   # Client secret (used for OAuth2)
BOT_OWNER=               # Your Discord user ID
```

The web dashboard is on by default and needs three generated secrets. Generate each and paste it into `.env`:

```sh
openssl rand -hex 32     # -> SESSION_SECRET
openssl rand -hex 32     # -> API_KEY_PEPPER
openssl rand -hex 32     # -> OAUTH_ENCRYPTION_KEY
```

The full variable reference lives on the [Configuration](Configuration) page.

## 2. Build and run

```sh
cargo build --release
./target/release/clouder
```

If `.env` is missing, clouder writes one from `.env.example`, prints an instruction, and exits straight
away, nothing else runs. Fill it in and start again.

With `.env` in place, the first run:

1. Creates the `data/` directory and `data/db.sqlite`.
2. Runs all migrations.
3. Registers slash commands globally.

## Feature matrix

Both `web` and `llm` are enabled by default. Compile a subset with `--no-default-features`:

```sh
cargo build --release                                          # bot + web + llm (default)
cargo build --release --no-default-features                    # bot only
cargo build --release --no-default-features --features web     # bot + web
cargo build --release --no-default-features --features llm     # bot + llm
```

| Flag | Default | Effect |
|------|---------|--------|
| `web` | on | Enables `clouder-web`, starts the Axum dashboard alongside the bot |
| `llm` | on | Enables `clouder-llm`, activates `clouder-core/llm`, places `LlmClient` in `AppState` |

## Discord permissions

Grant the bot these permissions:

- Send Messages, Embed Links, Read Message History
- Manage Messages (`/purge`, media-only)
- Manage Roles (self-role assignment)
- Manage Channels (channel commands, media-only)

## Gateway intents

Enable these intents for the application:

- `GUILDS`
- `GUILD_MESSAGES`
- `GUILD_MEMBERS` *(privileged, toggle in the Developer Portal)*
- `MESSAGE_CONTENT` *(privileged, toggle in the Developer Portal)*

## Next steps

- Tune behavior on the [Configuration](Configuration) page.
- See what each feature does on the [Features](Features) page.
- Set up the dashboard on the [Web Dashboard](Web-Dashboard) page.
