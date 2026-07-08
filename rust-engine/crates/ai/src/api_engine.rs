//! OpenAI-compatible API inference engine.

use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use llm_core::Result;

use crate::inference::{InferenceEngine, InferenceOptions, InferenceResult};

/// Configuration for the API inference engine.
#[derive(Clone, Serialize, Deserialize)]
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

impl fmt::Debug for APIConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("APIConfig")
            .field("base_url", &redact_sensitive_text(&self.base_url))
            .field("api_key", &redacted_secret(&self.api_key))
            .field("model", &self.model)
            .field("max_tokens", &self.max_tokens)
            .field("temperature", &self.temperature)
            .field("top_p", &self.top_p)
            .field("timeout_seconds", &self.timeout_seconds)
            .field("headers", &redacted_headers(&self.headers))
            .finish()
    }
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

        let mut client_builder =
            Client::builder().timeout(std::time::Duration::from_secs(self.config.timeout_seconds));

        // Add custom headers
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.config.api_key).parse().map_err(
                |e: reqwest::header::InvalidHeaderValue| {
                    llm_core::EngineError::config("api_key", format!("Invalid API key: {e}"))
                },
            )?,
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
            llm_core::EngineError::config(
                "http_client",
                format!("Failed to create HTTP client: {e}"),
            )
        })?);
        self.initialized = true;

        info!("API engine initialized successfully");
        Ok(())
    }

    async fn infer(&self, prompt: &str, options: &InferenceOptions) -> Result<InferenceResult> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| llm_core::EngineError::inference("API", "Engine not initialized"))?;

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
        let response = client.post(&url).json(&request).send().await.map_err(|e| {
            llm_core::EngineError::inference(
                "API",
                redact_sensitive_text(&format!("Request failed: {e}")),
            )
        })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let safe_body = redact_sensitive_text(&body);
            error!("API returned status {}: {}", status, safe_body);
            return Ok(InferenceResult {
                text: String::new(),
                success: false,
                error: Some(format!("API error {status}: {safe_body}")),
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
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| llm_core::EngineError::inference("API", "Engine not initialized"))?;

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
        let response = client.post(&url).json(&request).send().await.map_err(|e| {
            llm_core::EngineError::inference(
                "API",
                redact_sensitive_text(&format!("Request failed: {e}")),
            )
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let safe_body = redact_sensitive_text(&body);
            return Ok(InferenceResult {
                text: String::new(),
                success: false,
                error: Some(format!("API error {status}: {safe_body}")),
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

fn redacted_secret(value: &str) -> String {
    if value.trim().is_empty() {
        String::new()
    } else {
        "<redacted>".to_string()
    }
}

fn redacted_headers(headers: &HashMap<String, String>) -> HashMap<String, String> {
    headers
        .iter()
        .map(|(key, value)| {
            let redacted = if is_sensitive_header_name(key) {
                redacted_secret(value)
            } else {
                redact_sensitive_text(value)
            };
            (key.clone(), redacted)
        })
        .collect()
}

fn is_sensitive_header_name(name: &str) -> bool {
    let normalized = name.trim().to_ascii_lowercase();
    normalized == "authorization"
        || normalized == "proxy-authorization"
        || normalized == "x-api-key"
        || normalized == "api-key"
        || normalized == "xi-api-key"
        || normalized == "ocp-apim-subscription-key"
        || normalized.contains("token")
        || normalized.contains("secret")
        || normalized.contains("password")
}

fn redact_sensitive_text(text: &str) -> String {
    let token_redacted = redact_token_like_values(text);
    redact_secret_assignments(&token_redacted)
}

fn redact_token_like_values(text: &str) -> String {
    let mut redacted = String::with_capacity(text.len());
    let mut cursor = 0;
    while cursor < text.len() {
        if let Some((prefix, min_len)) = token_prefix_at(text, cursor) {
            let token_end = token_end(text, cursor);
            let body_len = text[cursor + prefix.len()..token_end]
                .chars()
                .filter(|ch| is_token_char(*ch))
                .count();
            if body_len >= min_len {
                redacted.push_str("<redacted>");
                cursor = token_end;
                continue;
            }
        }

        let ch = text[cursor..]
            .chars()
            .next()
            .expect("cursor at char boundary");
        redacted.push(ch);
        cursor += ch.len_utf8();
    }

    redacted
}

fn token_prefix_at(text: &str, cursor: usize) -> Option<(&'static str, usize)> {
    let rest = &text[cursor..];
    for (prefix, min_len) in [("github_pat_", 20), ("ghp_", 20), ("sk-", 20)] {
        if rest.starts_with(prefix) {
            return Some((prefix, min_len));
        }
    }
    None
}

fn token_end(text: &str, cursor: usize) -> usize {
    text[cursor..]
        .char_indices()
        .find_map(|(offset, ch)| (!is_token_char(ch)).then_some(cursor + offset))
        .unwrap_or(text.len())
}

fn is_token_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
}

fn redact_secret_assignments(text: &str) -> String {
    let mut redacted = text.to_string();
    for key in [
        "api_key",
        "apiKey",
        "access_token",
        "accessToken",
        "token",
        "secret",
        "password",
        "authorization",
    ] {
        redacted = redact_assignment_values(&redacted, key);
    }
    redacted
}

fn redact_assignment_values(text: &str, key: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut cursor = 0;

    while let Some(relative) = text[cursor..].find(key) {
        let key_start = cursor + relative;
        let key_end = key_start + key.len();
        if !is_key_boundary(text, key_start, key_end) {
            result.push_str(&text[cursor..key_end]);
            cursor = key_end;
            continue;
        }

        let Some((value_start, value_end)) = assignment_value_span(text, key_start, key_end) else {
            result.push_str(&text[cursor..key_end]);
            cursor = key_end;
            continue;
        };

        result.push_str(&text[cursor..value_start]);
        result.push_str("<redacted>");
        cursor = value_end;
    }

    result.push_str(&text[cursor..]);
    result
}

fn is_key_boundary(text: &str, start: usize, end: usize) -> bool {
    let before = text[..start].chars().next_back();
    let after = text[end..].chars().next();
    !before.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        && !after.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn assignment_value_span(text: &str, key_start: usize, key_end: usize) -> Option<(usize, usize)> {
    let mut idx = skip_ws(text, key_end);

    if let Some(quote) = text[..key_start]
        .chars()
        .next_back()
        .filter(|ch| matches!(ch, '"' | '\''))
    {
        if text[idx..].starts_with(quote) {
            idx += quote.len_utf8();
            idx = skip_ws(text, idx);
        }
    }

    if let Some(quote) = text[idx..]
        .chars()
        .next()
        .filter(|ch| matches!(ch, '"' | '\''))
    {
        idx += quote.len_utf8();
        idx = skip_ws(text, idx);
    }

    let separator = text[idx..].chars().next()?;
    if !matches!(separator, ':' | '=') {
        return None;
    }
    idx += separator.len_utf8();
    idx = skip_ws(text, idx);

    if let Some(quote) = text[idx..]
        .chars()
        .next()
        .filter(|ch| matches!(ch, '"' | '\''))
    {
        let value_start = idx + quote.len_utf8();
        let value_end = text[value_start..]
            .char_indices()
            .find_map(|(offset, ch)| (ch == quote).then_some(value_start + offset))
            .unwrap_or(text.len());
        return Some((value_start, value_end));
    }

    let value_start = idx;
    let value_end = text[value_start..]
        .char_indices()
        .find_map(|(offset, ch)| {
            (ch.is_whitespace() || matches!(ch, '&' | ',' | '}' | ']' | ';'))
                .then_some(value_start + offset)
        })
        .unwrap_or(text.len());
    (value_start < value_end).then_some((value_start, value_end))
}

fn skip_ws(text: &str, start: usize) -> usize {
    text[start..]
        .char_indices()
        .find_map(|(offset, ch)| (!ch.is_whitespace()).then_some(start + offset))
        .unwrap_or(text.len())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_config_debug_redacts_api_key_and_sensitive_headers() {
        let api_key = format!("sk-{}", "A".repeat(24));
        let header_token = format!("Bearer {}", api_key);
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), header_token.clone());
        headers.insert("X-Trace-Id".to_string(), "trace-123".to_string());

        let config = APIConfig {
            base_url: format!("https://example.test/v1?api_key={api_key}"),
            api_key: api_key.clone(),
            model: "test-model".to_string(),
            headers,
            ..Default::default()
        };

        let debug = format!("{config:?}");

        assert!(!debug.contains(&api_key));
        assert!(!debug.contains(&header_token));
        assert!(debug.contains("<redacted>"));
        assert!(debug.contains("trace-123"));
        assert!(debug.contains("test-model"));
    }

    #[test]
    fn api_error_redaction_handles_token_like_values_and_secret_assignments() {
        let openai_key = format!("sk-{}", "B".repeat(24));
        let github_key = format!("github_pat_{}", "C".repeat(24));
        let plain_secret = "plain-secret-value";
        let body = format!(
            r#"{{"error":"bad key","api_key":"{plain_secret}","url":"https://example.test?access_token={github_key}","authorization":"Bearer {openai_key}"}}"#
        );

        let redacted = redact_sensitive_text(&body);

        assert!(!redacted.contains(&openai_key));
        assert!(!redacted.contains(&github_key));
        assert!(!redacted.contains(plain_secret));
        assert!(redacted.contains("<redacted>"));
        assert!(redacted.contains("bad key"));
        assert!(redacted.contains("https://example.test?access_token=<redacted>"));
    }
}
