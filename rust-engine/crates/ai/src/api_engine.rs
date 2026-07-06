//! OpenAI-compatible API inference engine.

use std::collections::HashMap;
use std::time::Instant;

use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use llm_core::Result;

use crate::inference::{InferenceEngine, InferenceOptions, InferenceResult};

/// Configuration for the API inference engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIConfig {
    /// Base URL of the API (e.g., "https://api.openai.com/v1").
    pub base_url: String,
    /// API key for authentication.
    pub api_key: String,
    /// Model identifier (e.g., "gpt-3.5-turbo").
    pub model: String,
    /// Default maximum tokens.
    pub max_tokens: u32,
    /// Default temperature.
    pub temperature: f32,
    /// Default top_p.
    pub top_p: f32,
    /// Request timeout in seconds.
    pub timeout_seconds: u64,
    /// Additional headers to send with requests.
    pub headers: HashMap<String, String>,
}

impl Default for APIConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-3.5-turbo".to_string(),
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            timeout_seconds: 60,
            headers: HashMap::new(),
        }
    }
}

// --- API Request/Response types ---

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
    top_p: f32,
    stream: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stop: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: Option<String>,
}

#[derive(Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct StreamDelta {
    content: Option<String>,
}

/// OpenAI-compatible API inference engine.
pub struct APIEngine {
    config: APIConfig,
    client: Option<Client>,
    initialized: bool,
}

impl APIEngine {
    /// Create a new API engine with the given configuration.
    pub fn new(config: APIConfig) -> Self {
        Self {
            config,
            client: None,
            initialized: false,
        }
    }
}

#[async_trait]
impl InferenceEngine for APIEngine {
    fn name(&self) -> &str {
        "API"
    }

    fn is_ready(&self) -> bool {
        self.initialized
    }

    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing API engine with model: {}", self.config.model);

        let mut client_builder = Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds));

        // Add custom headers
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.config.api_key)
                .parse()
                .map_err(|e: reqwest::header::InvalidHeaderValue| {
                    llm_core::EngineError::config("api_key", format!("Invalid API key: {e}"))
                })?,
        );
        for (key, value) in &self.config.headers {
            if let (Ok(name), Ok(val)) = (
                reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                reqwest::header::HeaderValue::from_str(value),
            ) {
                headers.insert(name, val);
            }
        }
        client_builder = client_builder.default_headers(headers);

        self.client = Some(client_builder.build().map_err(|e| {
            llm_core::EngineError::config("http_client", format!("Failed to create HTTP client: {e}"))
        })?);
        self.initialized = true;

        info!("API engine initialized successfully");
        Ok(())
    }

    async fn infer(&self, prompt: &str, options: &InferenceOptions) -> Result<InferenceResult> {
        let client = self.client.as_ref().ok_or_else(|| {
            llm_core::EngineError::inference("API", "Engine not initialized")
        })?;

        let start = Instant::now();
        debug!("Sending inference request to API");

        // Parse prompt into messages (system/user split)
        let messages = parse_prompt_to_messages(prompt);

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            top_p: options.top_p,
            stream: false,
            stop: options.stop_sequences.clone(),
        };

        let url = format!("{}/chat/completions", self.config.base_url);
        let response = client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| llm_core::EngineError::inference("API", format!("Request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            error!("API returned status {}: {}", status, body);
            return Ok(InferenceResult {
                text: String::new(),
                success: false,
                error: Some(format!("API error {status}: {body}")),
                duration_ms: start.elapsed().as_millis() as u64,
                tokens_generated: 0,
            });
        }

        let chat_response: ChatResponse = response.json().await.map_err(|e| {
            llm_core::EngineError::inference("API", format!("Failed to parse response: {e}"))
        })?;

        let text = chat_response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        let duration = start.elapsed().as_millis() as u64;
        debug!("API inference completed in {}ms", duration);

        Ok(InferenceResult {
            text,
            success: true,
            error: None,
            duration_ms: duration,
            tokens_generated: 0, // API doesn't always report this
        })
    }

    async fn infer_stream(
        &self,
        prompt: &str,
        options: &InferenceOptions,
        on_chunk: Box<dyn Fn(String) + Send + 'static>,
    ) -> Result<InferenceResult> {
        let client = self.client.as_ref().ok_or_else(|| {
            llm_core::EngineError::inference("API", "Engine not initialized")
        })?;

        let start = Instant::now();
        debug!("Sending streaming inference request to API");

        let messages = parse_prompt_to_messages(prompt);

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            top_p: options.top_p,
            stream: true,
            stop: options.stop_sequences.clone(),
        };

        let url = format!("{}/chat/completions", self.config.base_url);
        let response = client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| llm_core::EngineError::inference("API", format!("Request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Ok(InferenceResult {
                text: String::new(),
                success: false,
                error: Some(format!("API error {status}: {body}")),
                duration_ms: start.elapsed().as_millis() as u64,
                tokens_generated: 0,
            });
        }

        let mut full_text = String::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| {
                llm_core::EngineError::inference("API", format!("Stream read error: {e}"))
            })?;

            let text = String::from_utf8_lossy(&chunk);
            for line in text.lines() {
                let line = line.trim();
                if !line.starts_with("data:") || line == "data: [DONE]" {
                    continue;
                }
                let data = line.trim_start_matches("data:").trim();
                if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                    for choice in chunk.choices {
                        if let Some(content) = choice.delta.content {
                            full_text.push_str(&content);
                            on_chunk(content);
                        }
                    }
                }
            }
        }

        let duration = start.elapsed().as_millis() as u64;
        debug!("API streaming inference completed in {}ms", duration);

        Ok(InferenceResult {
            text: full_text,
            success: true,
            error: None,
            duration_ms: duration,
            tokens_generated: 0,
        })
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.client = None;
        self.initialized = false;
        info!("API engine shut down");
        Ok(())
    }
}

/// Parse a prompt string into chat messages.
///
/// Expects format with `[System]`, `[User]`, `[Assistant]` markers.
fn parse_prompt_to_messages(prompt: &str) -> Vec<ChatMessage> {
    let mut messages = Vec::new();
    let mut current_role = "user";
    let mut current_content = String::new();

    for line in prompt.lines() {
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case("[System]") {
            if !current_content.is_empty() {
                messages.push(ChatMessage {
                    role: current_role.to_string(),
                    content: current_content.trim().to_string(),
                });
                current_content.clear();
            }
            current_role = "system";
        } else if trimmed.eq_ignore_ascii_case("[User]") {
            if !current_content.is_empty() {
                messages.push(ChatMessage {
                    role: current_role.to_string(),
                    content: current_content.trim().to_string(),
                });
                current_content.clear();
            }
            current_role = "user";
        } else if trimmed.eq_ignore_ascii_case("[Assistant]") {
            if !current_content.is_empty() {
                messages.push(ChatMessage {
                    role: current_role.to_string(),
                    content: current_content.trim().to_string(),
                });
                current_content.clear();
            }
            current_role = "assistant";
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    if !current_content.is_empty() {
        messages.push(ChatMessage {
            role: current_role.to_string(),
            content: current_content.trim().to_string(),
        });
    }

    messages
}
