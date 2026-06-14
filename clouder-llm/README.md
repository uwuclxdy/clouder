# clouder-llm

Optional OpenAI-compatible LLM client. Gated behind the `llm` feature flag at the workspace level. When
enabled, an `LlmClient` instance is placed in `clouder-core`'s `AppState` and used by the bot's `@mention`
event handler to generate conversational responses.

## Public API

Defined in `src/openai.rs` and re-exported from the crate root (`pub use openai::{ChatMessage, LlmClient}`).

### Exported types

```rust
pub struct ChatMessage {
    pub role: String,      // "system", "user", "assistant"
    pub content: String,
}
```

`ChatRequest`, `ChatResponse`, and `Choice` are internal (private) wire types, not part of the public API.

### LlmClient

```rust
pub struct LlmClient { /* reqwest::Client, base_url, api_key, in-memory cooldowns */ }
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `(base_url: String, api_key: String, timeout_seconds: u64) -> Self` | Builds a reqwest client with the given timeout |
| `generate` | `(&self, model: &str, messages: Vec<ChatMessage>, temperature: f32, max_tokens: u32, stop: Option<&str>) -> Result<String>` | POSTs to `{base_url}/chat/completions`, returns the first choice's cleaned content. Sends the `Authorization: Bearer` header only when `api_key` is non-empty |
| `check_and_update_cooldown` | `(&self, user_id: u64, cooldown_duration: Duration) -> bool` | Returns `true` if the user is allowed (no active cooldown) and records the request; evicts expired entries on each call |

## Design notes

- **No tokio dependency.** Runs on the caller's async runtime. Depends only on reqwest, serde, anyhow, and tracing.
- **In-memory cooldowns.** Per-user request throttling in `Arc<Mutex<HashMap<u64, Instant>>>`. Not persisted across restarts; expired entries are pruned on each check.
- **Response cleaning.** Strips leaked end-of-sequence tokens (`</s>`, `<|endoftext|>`, `<|im_end|>`, `<|eot_id|>`, `<|end_of_text|>`, `<|end|>`, `<|EOT|>`) for compatibility with open-source models.
- **OpenAI-compatible.** Works with any API that implements `/chat/completions` (OpenAI, Ollama, LM Studio, and similar).

## Integration path

```
Cargo.toml (workspace root)
  features.llm = ["clouder-llm", "clouder-core/llm"]

clouder-core/config.rs
  AppState::new() instantiates LlmClient when config.llm.provider is Some
  AppState.llm_client: Option<clouder_llm::LlmClient>   (under #[cfg(feature = "llm")])

clouder/src/events/bot_mentioned.rs
  Uses data.llm_client to generate responses on @mention
```
