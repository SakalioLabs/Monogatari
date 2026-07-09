//! ONNX Runtime inference engine for local LLM execution.

use std::path::PathBuf;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::info;

use llm_core::Result;

use crate::inference::{InferenceEngine, InferenceOptions, InferenceResult};

/// Message returned when this build has ONNX configuration support but no ONNX Runtime executor.
pub const ONNX_RUNTIME_UNAVAILABLE_MESSAGE: &str =
    "ONNX Runtime execution is not linked in this build; local ONNX inference is unavailable. Configure the API backend or enable an ONNX Runtime integration before using ONNX mode.";

/// Configuration for the ONNX inference engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Path to the ONNX model file.
    pub model_path: PathBuf,
    /// Path to the tokenizer vocabulary file.
    pub tokenizer_path: PathBuf,
    /// Maximum sequence length the model supports.
    pub max_sequence_length: usize,
    /// Vocabulary size.
    pub vocab_size: usize,
    /// Whether to use DirectML for GPU acceleration.
    pub use_directml: bool,
    /// GPU device ID for DirectML.
    pub device_id: i32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::new(),
            tokenizer_path: PathBuf::new(),
            max_sequence_length: 2048,
            vocab_size: 32000,
            use_directml: true,
            device_id: 0,
        }
    }
}

/// A simple tokenizer that loads vocabulary from JSON.
pub struct Tokenizer {
    vocab: Vec<String>,
    token_to_id: std::collections::HashMap<String, u32>,
}

impl Tokenizer {
    /// Load a tokenizer from a JSON vocabulary file.
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let vocab: Vec<String> = serde_json::from_str(&content)?;

        let mut token_to_id = std::collections::HashMap::new();
        for (id, token) in vocab.iter().enumerate() {
            token_to_id.insert(token.clone(), id as u32);
        }

        Ok(Self { vocab, token_to_id })
    }

    /// Encode text into token IDs using greedy longest-match.
    pub fn encode(&self, text: &str) -> Vec<u32> {
        let mut tokens = Vec::new();
        let mut pos = 0;
        let chars: Vec<char> = text.chars().collect();

        while pos < chars.len() {
            let mut best_match: Option<(usize, u32)> = None;

            // Try matching from longest to shortest
            for len in (1..=(chars.len() - pos).min(32)).rev() {
                let candidate: String = chars[pos..pos + len].iter().collect();
                if let Some(&id) = self.token_to_id.get(&candidate) {
                    best_match = Some((len, id));
                    break;
                }
            }

            if let Some((len, id)) = best_match {
                tokens.push(id);
                pos += len;
            } else {
                // Unknown token - skip character
                pos += 1;
            }
        }

        tokens
    }

    /// Decode token IDs back into text.
    pub fn decode(&self, tokens: &[u32]) -> String {
        tokens
            .iter()
            .filter_map(|&id| self.vocab.get(id as usize))
            .cloned()
            .collect::<Vec<_>>()
            .join("")
    }
}

/// ONNX Runtime inference engine for local LLM execution.
pub struct ONNXEngine {
    config: ModelConfig,
    initialized: bool,
}

impl ONNXEngine {
    /// Create a new ONNX engine with the given configuration.
    pub fn new(config: ModelConfig) -> Self {
        Self {
            config,
            initialized: false,
        }
    }

    fn runtime_unavailable_error() -> llm_core::EngineError {
        llm_core::EngineError::inference("ONNX", ONNX_RUNTIME_UNAVAILABLE_MESSAGE)
    }
}

#[async_trait]
impl InferenceEngine for ONNXEngine {
    fn name(&self) -> &str {
        "ONNX"
    }

    fn is_ready(&self) -> bool {
        self.initialized
    }

    async fn initialize(&mut self) -> Result<()> {
        info!(
            "Initializing ONNX engine with model: {}",
            self.config.model_path.display()
        );

        // Verify model file exists
        if !self.config.model_path.exists() {
            return Err(llm_core::EngineError::config(
                "model_path",
                format!("Model file not found: {}", self.config.model_path.display()),
            ));
        }

        // Verify tokenizer file exists
        if !self.config.tokenizer_path.exists() {
            return Err(llm_core::EngineError::config(
                "tokenizer_path",
                format!(
                    "Tokenizer file not found: {}",
                    self.config.tokenizer_path.display()
                ),
            ));
        }

        let _tokenizer = Tokenizer::from_file(&self.config.tokenizer_path)?;

        self.initialized = false;
        Err(Self::runtime_unavailable_error())
    }

    async fn infer(&self, _prompt: &str, _options: &InferenceOptions) -> Result<InferenceResult> {
        Err(Self::runtime_unavailable_error())
    }

    async fn infer_stream(
        &self,
        prompt: &str,
        options: &InferenceOptions,
        on_chunk: Box<dyn Fn(String) + Send + 'static>,
    ) -> Result<InferenceResult> {
        let _ = on_chunk;
        self.infer(prompt, options).await
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.initialized = false;
        info!("ONNX engine shut down");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_model_dir(label: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "monogatari-onnx-{label}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("model.onnx"), b"placeholder model").unwrap();
        std::fs::write(dir.join("tokenizer.json"), br#"["hello","world"]"#).unwrap();
        dir
    }

    fn test_config(dir: &std::path::Path) -> ModelConfig {
        ModelConfig {
            model_path: dir.join("model.onnx"),
            tokenizer_path: dir.join("tokenizer.json"),
            ..Default::default()
        }
    }

    #[test]
    fn onnx_initialize_reports_runtime_unavailable_without_ready_state() {
        let dir = temp_model_dir("initialize");
        let mut engine = ONNXEngine::new(test_config(&dir));
        let rt = tokio::runtime::Runtime::new().unwrap();

        let error = rt.block_on(engine.initialize()).unwrap_err();

        assert!(error.to_string().contains(ONNX_RUNTIME_UNAVAILABLE_MESSAGE));
        assert!(!engine.is_ready());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn onnx_infer_reports_runtime_unavailable_without_placeholder_success() {
        let dir = temp_model_dir("infer");
        let engine = ONNXEngine::new(test_config(&dir));
        let rt = tokio::runtime::Runtime::new().unwrap();

        let error = rt
            .block_on(engine.infer("hello", &InferenceOptions::default()))
            .unwrap_err();

        assert!(error.to_string().contains(ONNX_RUNTIME_UNAVAILABLE_MESSAGE));
        assert!(!error.to_string().contains("placeholder"));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn onnx_stream_reports_runtime_unavailable_without_chunks() {
        let dir = temp_model_dir("stream");
        let engine = ONNXEngine::new(test_config(&dir));
        let emitted = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let emitted_for_callback = emitted.clone();
        let rt = tokio::runtime::Runtime::new().unwrap();

        let error = rt
            .block_on(engine.infer_stream(
                "hello",
                &InferenceOptions::default(),
                Box::new(move |chunk| emitted_for_callback.lock().unwrap().push(chunk)),
            ))
            .unwrap_err();

        assert!(error.to_string().contains(ONNX_RUNTIME_UNAVAILABLE_MESSAGE));
        assert!(emitted.lock().unwrap().is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
