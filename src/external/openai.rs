use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, warn};

#[derive(Debug, Serialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ChatMessage,
}

#[derive(Clone)]
pub struct OpenAIClient {
    client: Client,
    base_url: String,
    api_key: String,
    cooldowns: Arc<Mutex<HashMap<u64, Instant>>>,
}

impl OpenAIClient {
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
        let url = format!("{}/chat/completions", self.base_url);

        let stop_sequences = stop
            .filter(|s| !s.trim().is_empty())
            .map(|s| vec![s.to_string()]);

        let request = OpenAIRequest {
            model: model.to_string(),
            messages,
            temperature,
            max_tokens,
            stop: stop_sequences,
        };

        debug!("Sending request to OpenAI: {} with model {}", url, model);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("OpenAI API error: {}", error_text);
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        let openai_response: OpenAIResponse = response.json().await?;

        if openai_response.choices.is_empty() {
            warn!("OpenAI returned empty choices array");
            return Ok("No response generated.".to_string());
        }

        let content = openai_response.choices[0].message.content.clone();
        let cleaned_content = clean_response_tokens(&content);

        debug!("OpenAI response: {}", cleaned_content);
        Ok(cleaned_content)
    }

    pub fn check_and_update_cooldown(&self, user_id: u64, cooldown_duration: Duration) -> bool {
        let mut cooldowns = self.cooldowns.lock().unwrap();
        let now = Instant::now();

        if let Some(&last_request) = cooldowns.get(&user_id)
            && now.duration_since(last_request) < cooldown_duration {
                return false; // User is still on cooldown
            }

        cooldowns.insert(user_id, now);
        true
    }
}

/// Remove end-of-sequence tokens from the response text
fn clean_response_tokens(text: &str) -> String {
    text.replace("</s>", "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_end_of_sequence_tokens() {
        // Test that '</s>' tokens are properly removed
        let text_with_tokens = "Hello world</s>Test content</s>";
        let cleaned = clean_response_tokens(text_with_tokens);
        assert_eq!(cleaned, "Hello worldTest content");

        // Test that text without tokens remains unchanged
        let text_without_tokens = "Hello world";
        let cleaned = clean_response_tokens(text_without_tokens);
        assert_eq!(cleaned, "Hello world");

        // Test empty string
        let empty_text = "";
        let cleaned = clean_response_tokens(empty_text);
        assert_eq!(cleaned, "");

        // Test only tokens
        let only_tokens = "</s></s></s>";
        let cleaned = clean_response_tokens(only_tokens);
        assert_eq!(cleaned, "");
    }

    #[test]
    fn test_cooldown_functionality() {
        let client = OpenAIClient::new(
            "https://api.openai.com/v1".to_string(),
            "test-key".to_string(),
            30,
        );
        let user_id = 123456789;
        let cooldown_duration = Duration::from_secs(5);

        // First request should succeed
        assert!(client.check_and_update_cooldown(user_id, cooldown_duration));

        // Immediate second request should fail (cooldown active)
        assert!(!client.check_and_update_cooldown(user_id, cooldown_duration));

        // Different user should succeed
        let other_user_id = 987654321;
        assert!(client.check_and_update_cooldown(other_user_id, cooldown_duration));
    }

    #[test]
    fn test_stop_functionality() {
        // Test that stop sequences are properly handled in request structure
        let request_with_stop = OpenAIRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            temperature: 0.7,
            max_tokens: 100,
            stop: Some(vec!["STOP".to_string(), "END".to_string()]),
        };

        let request_without_stop = OpenAIRequest {
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
