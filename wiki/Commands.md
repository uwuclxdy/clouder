# Commands

Every slash command, with the Discord permission it requires. Commands marked **Anyone** are usable by
everyone in the server.

| Command | Description | Permission |
|---------|-------------|------------|
| `/about bot \| server \| user \| role \| channel` | Info and live stats (uptime, RAM, CPU, latency) | Anyone |
| `/help [category]` | List commands by category | Anyone |
| `/selfroles` | Link to the web dashboard for self-role setup | Manage Roles |
| `/purge <count \| message_id>` | Bulk-delete messages | Manage Messages |
| `/mediaonly <channel> [enabled]` | Toggle media-only mode on a channel | Manage Channels |
| `/channel delete \| clone_channel \| nuke` | Channel management | Manage Channels |
| `/reminders` | View active reminders | Anyone |
| `/hf latest \| trending` | Browse HuggingFace models | Anyone |
| `/github <user> [repo]` | GitHub user or repo stats | Anyone |
| `/gh-trending [period]` | Trending GitHub repos | Anyone |
| `/uwufy [user]` | Toggle uwuify on a user | Manage Guild |
| `/random` | Random number generator | Anyone |
| `/tinyfox animal \| progress` | Random animal pictures | Anyone |

## Notes

- `/selfroles` and the dashboard manage the same data. See [Web Dashboard](Web-Dashboard).
- `/mediaonly` and `/channel` need the bot to hold **Manage Channels**; `/purge` needs **Manage Messages**.
  See [Installation](Installation#discord-permissions) for the full permission set.
- Commands register globally on first run.
