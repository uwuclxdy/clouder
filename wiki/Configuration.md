# Configuration

clouder reads all configuration from environment variables, loaded from `.env` at startup
(see [`.env.example`](https://github.com/uwuclxdy/clouder/blob/main/.env.example)). This page is the
full reference. Defaults below are the **code fallbacks** applied when a variable is unset.

> [!NOTE]
> The shipped `.env.example` pre-fills some LLM values with opinionated, chattier choices
> (`LLM_MODEL=gpt-4o`, `LLM_MAX_TOKENS=30`, `LLM_TIMEOUT_SECONDS=60`) that differ from the code
> fallbacks listed here. Delete a line from `.env` to fall back to the value in the table.

## Required

| Variable | Description |
|----------|-------------|
| `DISCORD_TOKEN` | Bot token from the Discord Developer Portal |
| `DISCORD_CLIENT_ID` | Application / client ID (parsed as the application ID) |
| `DISCORD_CLIENT_SECRET` | Client secret, used for OAuth2 |
| `BOT_OWNER` | Discord user ID with owner-level bot permissions |
| `SESSION_SECRET` | Signs session cookies (HKDF-derived). At least 32 characters |
| `API_KEY_PEPPER` | Pepper for hashing dashboard API keys. At least 32 characters. Rotating it invalidates every API key; users regenerate from `/profile` |
| `OAUTH_ENCRYPTION_KEY` | AES-256 key encrypting Discord OAuth tokens at rest. Exactly 64 hex chars (32 bytes). Rotating it forces all users to re-authenticate |

> [!IMPORTANT]
> The three secrets are required for **every** build, not only when the `web` feature is on:
> `Config::from_env()` loads them unconditionally and exits if any is missing or too short.
> `SESSION_SECRET`, `API_KEY_PEPPER`, `OAUTH_ENCRYPTION_KEY`, and `DISCORD_CLIENT_SECRET` must all be
> distinct. Generate each with `openssl rand -hex 32`.

## Web dashboard

| Variable | Default | Description |
|----------|---------|-------------|
| `API_BASE` | `http://127.0.0.1:8080` | Public base URL, used for OAuth redirects |
| `WEB_BIND_ADDR` | `127.0.0.1:3000` | Address the server binds to |
| `DISCORD_REDIRECT_URI` | `{API_BASE}/auth/callback` | OAuth redirect URI (override only if needed) |

## Database

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `data/db.sqlite` | SQLite path. `.env.example` ships it with a `sqlite:` scheme prefix (`sqlite:data/db.sqlite`) |

## Appearance

| Variable | Default | Description |
|----------|---------|-------------|
| `EMBED_DEFAULT_COLOR` | `#FFFFFF` | Default embed color. Hex (`#RRGGBB`, `0xRRGGBB`) or decimal |

## Logging

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `warn,clouder=info,clouder_core=info,clouder_web=info,clouder_llm=info` | Per-crate log levels, read by `tracing_subscriber` (`trace`, `debug`, `info`, `warn`, `error`) |

## LLM integration

Set `LLM_PROVIDER` to enable. Leave it unset to disable LLM responses entirely.

| Variable | Default | Description |
|----------|---------|-------------|
| `LLM_PROVIDER` | unset | `openai` or `ollama`. Unset (or unknown) disables the feature |
| `LLM_BASE_URL` | `https://api.openai.com/v1` (openai) · `http://localhost:11434/v1` (ollama) | API endpoint |
| `LLM_API_KEY` | empty | Bearer token |
| `LLM_MODEL` | `gpt-3.5-turbo` (openai) · `llama3.2` (ollama) | Model name |
| `LLM_TEMPERATURE` | `0.7` | Sampling temperature |
| `LLM_MAX_TOKENS` | `1000` | Max tokens per response |
| `LLM_TIMEOUT_SECONDS` | `30` | Request timeout |
| `LLM_SYSTEM_PROMPT` | empty | System prompt prepended to every request |
| `LLM_STOP` | empty | Stop sequence |
| `LLM_ALLOWED_USERS` | empty | Comma-separated user IDs allowed to trigger replies in servers |
| `LLM_DM_ALLOWED_USERS` | empty | Comma-separated user IDs allowed to trigger replies in DMs |
| `LLM_NO_COOLDOWN_USERS` | empty | Comma-separated user IDs exempt from the per-user cooldown |

> [!NOTE]
> The client targets any OpenAI-compatible `/chat/completions` endpoint (OpenAI, Ollama, LM Studio, and
> similar). See the [Features](Features#llm-mentions) page for how mentions trigger a reply.

## GitHub API

| Variable | Default | Description |
|----------|---------|-------------|
| `GITHUB_TOKEN` | unset | Optional. Raises the GitHub API rate limit from 60/hr to 5000/hr |

## Scheduler

| Variable | Default | Description |
|----------|---------|-------------|
| `SCHEDULER_INTERVAL` | `60` | Reminder check interval, in seconds |
| `DEFAULT_TIMEZONE` | `UTC` | Fallback timezone for guilds with none set (IANA name, e.g. `America/New_York`) |
