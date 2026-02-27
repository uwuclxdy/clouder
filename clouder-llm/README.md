# clouder-llm

Optional OpenAI-compatible LLM client. Gated behind the `llm` feature flag at the workspace level. When enabled, an `OpenAIClient` instance is placed in `clouder-core`'s `AppState` and used by the bot's `@mention` event handler to generate conversational responses.

## Public API

All types are defined in `src/openai.rs` and re-exported from the crate root.

### Types

```rust
pub struct ChatMessage {
    pub role: String,      // "system", "user", "assistant"
    pub content: String,
}

pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub stop: Option<Vec<String>>,
}

pub struct OpenAIResponse {
    pub choices: Vec<Choice>,
}

pub struct Choice {
    pub message: ChatMessage,
}
```

### OpenAIClient

```rust
pub struct OpenAIClient { /* reqwest::Client, base_url, api_key, cooldowns */ }
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `(base_url, api_key, timeout_seconds) -> Self` | Builds a reqwest client with the given timeout |
| `generate` | `(&self, model, messages, temperature, max_tokens, stop) -> Result<String>` | POSTs to `{base_url}/chat/completions`, returns first choice content |
| `check_and_update_cooldown` | `(&self, user_id, cooldown_duration) -> bool` | Returns `true` if the user is allowed (no active cooldown), records the request timestamp |

## Design notes

- **No tokio dependency** -- uses the caller's async runtime. Only depends on reqwest, serde, anyhow, and tracing.
- **In-memory cooldowns** -- per-user request throttling stored in `Arc<Mutex<HashMap<u64, Instant>>>`. Not persisted across restarts.
- **Response cleaning** -- strips `</s>` end-of-sequence tokens from responses for compatibility with open-source models that leak special tokens.
- **OpenAI-compatible** -- works with any API that implements the `/chat/completions` endpoint (OpenAI, local models via LM Studio/Ollama, etc.).

## Integration path

```
Cargo.toml (workspace root)
  features.llm = ["clouder-llm", "clouder-core/llm"]

clouder-core/config.rs
  AppState::new() instantiates OpenAIClient when config.openai.enabled

clouder/src/events/bot_mentioned.rs
  Uses data.openai_client to generate responses on @mention
```
