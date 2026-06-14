# clouder Wiki

The full reference for **clouder**, a modular Discord bot written in Rust (Serenity + Poise) with an
Axum web dashboard, an OpenAI-compatible LLM client, and a reminder scheduler.

> This wiki is a read-only mirror of the in-repo [`wiki/`](https://github.com/uwuclxdy/clouder/tree/main/wiki)
> folder. Edit the files there; web edits are overwritten on the next sync. For the project overview,
> see the [README](https://github.com/uwuclxdy/clouder#readme).

## Pages

| Page | Covers |
|------|--------|
| [Installation](Installation) | Prerequisites, build, run, feature matrix, Discord permissions and intents |
| [Configuration](Configuration) | Every environment variable, secrets, LLM and scheduler settings |
| [Commands](Commands) | Full slash-command reference with required permissions |
| [Features](Features) | How each feature behaves, plus background tasks |
| [Web Dashboard](Web-Dashboard) | OAuth2 flow and REST API endpoints |
| [Architecture](Architecture) | Crates, module layout, `AppState`, dependency graph |
| [Database](Database) | Schema, tables, migrations, storage model |

## At a glance

| Crate | Role | Feature |
|-------|------|---------|
| `clouder` | Bot binary: runtime, commands, events, scheduler | always |
| `clouder-core` | Config, database, business logic, utilities | always |
| `clouder-llm` | OpenAI-compatible LLM client | `llm` |
| `clouder-web` | Axum web dashboard and REST API | `web` |

Both `web` and `llm` are enabled by default.
