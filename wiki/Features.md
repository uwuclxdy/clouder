# Features

What each feature does and how it behaves at runtime.

## LLM mentions

Responds to `@mentions` and replies using a configurable OpenAI-compatible provider.

- Whitelist-based: only user IDs in `LLM_ALLOWED_USERS` (or `LLM_DM_ALLOWED_USERS` for DMs) trigger a reply.
- Per-user cooldown, kept in memory and not persisted across restarts. IDs in `LLM_NO_COOLDOWN_USERS` are exempt.
- An `ai_retry` button lets the user regenerate a response.
- Responses are stripped of leaked end-of-sequence tokens (`</s>`, `<|im_end|>`, `<|eot_id|>`, `<|endoftext|>`, and others) for open-source model compatibility.

Configure under the LLM section of [Configuration](Configuration#llm-integration). Requires the `llm`
feature (on by default).

## Media-only channels

Auto-deletes non-media messages in configured channels.

- Per-channel content rules: links, attachments, GIFs, stickers can each be allowed or denied.
- Content detection inspects attachments, embeds, sticker items, and URLs (including Tenor/Giphy GIF links).
- Toggle per channel with `/mediaonly` or from the dashboard.

## UwUify

Rewrites messages from toggled users into uwu-speak.

- Toggle per user with `/uwufy`.
- State is stored per `(guild, user)`.

## Welcome / goodbye

Sends configurable messages when members join or leave.

- Separate config for welcome and goodbye: enabled flag, channel, message type (embed or text), and content.
- Embed builder supports title, description, color, footer, thumbnail, image, and timestamp.
- Placeholders are replaced at send time: `{user}`, `{server}`, `{member_count}`, and more.
- Send a test message from the dashboard to preview the result.

## Self-role buttons

Button-driven role assignment, configured from the dashboard.

- Selection type is `radio` (single) or `multiple`.
- Per-role cooldowns prevent rapid toggling.
- Deploys a Discord message with one button per role; edits in place when the config changes.

## Message cleanup

Removes orphaned self-role data when its message is deleted, keeping the database consistent with Discord.

## Reminders

Scheduled reminders delivered to a channel or via DM.

- Per-guild reminder configs with subscriptions and ping roles.
- Timezone-aware; falls back to `DEFAULT_TIMEZONE` when a guild has none set.
- The scheduler debounces (~55s) so each reminder fires once per due window.
- View active reminders with `/reminders`.

## Background tasks

Two long-running tasks run alongside the bot:

| Task | Cadence | What it does |
|------|---------|--------------|
| Cleanup | every 5 minutes | Purges expired self-role cooldowns and expired dashboard sessions |
| Reminder scheduler | `SCHEDULER_INTERVAL` (default 60s) | Checks for due reminders and sends them, with a ~55s debounce |
| Web session sweep | every 15 minutes | The dashboard separately deletes expired sessions |
