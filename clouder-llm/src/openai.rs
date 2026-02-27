use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, warn};

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    stop: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    base_url: String,
    api_key: String,
    cooldowns: Arc<Mutex<HashMap<u64, Instant>>>,
}

impl LlmClient {
    pub fn new(base_url: String, api_key: String, timeout_seconds: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url,
            api_key,
            cooldowns: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn generate(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
        temperature: f32,
        max_tokens: u32,
        stop: Option<&str>,
    ) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let stop_sequences = stop
            .filter(|s| !s.trim().is_empty())
            .map(|s| vec![s.to_string()]);

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            temperature,
            max_tokens,
            stop: stop_sequences,
        };

        debug!("llm request: {} model {}", url, model);

        let mut builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_vec(&request)?);

        if !self.api_key.is_empty() {
            builder = builder.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let response = builder.send().await?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            error!("llm API error: {}", error_text);
            return Err(anyhow::anyhow!("llm API error: {}", error_text));
        }

        let text = response.text().await?;
        let chat_response: ChatResponse = serde_json::from_str(&text)?;

        if chat_response.choices.is_empty() {
            warn!("llm returned empty choices");
            return Ok("no response generated.".to_string());
        }

        let content = chat_response.choices[0].message.content.clone();
        let cleaned = clean_response_tokens(&content);

        debug!("llm response: {}", cleaned);
        Ok(cleaned)
    }

    pub fn check_and_update_cooldown(&self, user_id: u64, cooldown_duration: Duration) -> bool {
        let mut cooldowns = self.cooldowns.lock().expect("cooldowns lock poisoned");
        let now = Instant::now();

        // Evict expired entries to prevent unbounded growth
        cooldowns.retain(|_, last| now.duration_since(*last) < cooldown_duration);

        if let Some(&last_request) = cooldowns.get(&user_id)
            && now.duration_since(last_request) < cooldown_duration
        {
            return false;
        }

        cooldowns.insert(user_id, now);
        true
    }
}

fn clean_response_tokens(text: &str) -> String {
    // Strip common EOS tokens emitted by Ollama, LLaMA, Mistral, and other providers
    const EOS_TOKENS: &[&str] = &[
        "</s>",
        "<|endoftext|>",
        "<|im_end|>",
        "<|eot_id|>",
        "<|end_of_text|>",
        "<|end|>",
        "<|EOT|>",
    ];
    let mut result = text.to_string();
    for token in EOS_TOKENS {
        result = result.replace(token, "");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_end_of_sequence_tokens() {
        assert_eq!(
            clean_response_tokens("Hello world</s>Test content</s>"),
            "Hello worldTest content"
        );
        assert_eq!(clean_response_tokens("Hello world"), "Hello world");
        assert_eq!(clean_response_tokens(""), "");
        assert_eq!(clean_response_tokens("</s></s></s>"), "");
        assert_eq!(clean_response_tokens("Hello<|endoftext|>"), "Hello");
        assert_eq!(clean_response_tokens("Hello<|im_end|>"), "Hello");
        assert_eq!(clean_response_tokens("Hello<|eot_id|>"), "Hello");
        assert_eq!(clean_response_tokens("Hello<|end_of_text|>"), "Hello");
        assert_eq!(clean_response_tokens("Hello<|end|>"), "Hello");
        assert_eq!(clean_response_tokens("Hello<|EOT|>"), "Hello");
    }

    #[test]
    fn test_cooldown_functionality() {
        let client = LlmClient::new(
            "https://api.openai.com/v1".to_string(),
            "test-key".to_string(),
            30,
        );
        let user_id = 123456789;
        let cooldown_duration = Duration::from_secs(5);

        assert!(client.check_and_update_cooldown(user_id, cooldown_duration));
        assert!(!client.check_and_update_cooldown(user_id, cooldown_duration));

        let other_user_id = 987654321;
        assert!(client.check_and_update_cooldown(other_user_id, cooldown_duration));
    }

    #[test]
    fn test_stop_functionality() {
        let request_with_stop = ChatRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            temperature: 0.7,
            max_tokens: 100,
            stop: Some(vec!["STOP".to_string(), "END".to_string()]),
        };

        let request_without_stop = ChatRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            temperature: 0.7,
            max_tokens: 100,
            stop: None,
        };

        // Test that both serialize correctly
        assert!(serde_json::to_string(&request_with_stop).is_ok());
        assert!(serde_json::to_string(&request_without_stop).is_ok());

        // Test that stop field is handled properly
        assert!(request_with_stop.stop.is_some());
        assert!(request_without_stop.stop.is_none());

        // Test that stop sequences contain expected values
        if let Some(ref stop_sequences) = request_with_stop.stop {
            assert_eq!(stop_sequences.len(), 2);
            assert!(stop_sequences.contains(&"STOP".to_string()));
            assert!(stop_sequences.contains(&"END".to_string()));
        }
    }

    #[test]
    fn test_chat_message_structure() {
        // Test that ChatMessage serializes correctly
        let system_message = ChatMessage {
            role: "system".to_string(),
            content: "You are a helpful assistant".to_string(),
        };

        let user_message = ChatMessage {
            role: "user".to_string(),
            content: "Hello, how are you?".to_string(),
        };

        // Test serialization
        assert!(serde_json::to_string(&system_message).is_ok());
        assert!(serde_json::to_string(&user_message).is_ok());

        // Test that fields are correctly set
        assert_eq!(system_message.role, "system");
        assert_eq!(user_message.role, "user");
        assert_eq!(system_message.content, "You are a helpful assistant");
        assert_eq!(user_message.content, "Hello, how are you?");
    }
}
