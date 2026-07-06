//! Core inference types and traits.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use llm_core::Result;

/// Options for controlling LLM inference behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceOptions {
    /// Maximum number of tokens to generate.
    pub max_tokens: u32,
    /// Sampling temperature (0.0 = deterministic, higher = more random).
    pub temperature: f32,
    /// Top-p (nucleus) sampling threshold.
    pub top_p: f32,
    /// Top-k sampling threshold.
    pub top_k: u32,
    /// Repetition penalty to reduce repetitive output.
    pub repetition_penalty: f32,
    /// Sequences that stop generation when encountered.
    pub stop_sequences: Vec<String>,
}

impl Default for InferenceOptions {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 50,
            repetition_penalty: 1.1,
            stop_sequences: vec![],
        }
    }
}

/// Result from an LLM inference call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    /// The generated text.
    pub text: String,
    /// Whether the inference was successful.
    pub success: bool,
    /// Error message if inference failed.
    pub error: Option<String>,
    /// Time taken for inference in milliseconds.
    pub duration_ms: u64,
    /// Number of tokens generated.
    pub tokens_generated: u32,
}

/// Trait for LLM inference engines (API or local ONNX).
#[async_trait]
pub trait InferenceEngine: Send + Sync {
    /// Human-readable name of this engine.
    fn name(&self) -> &str;

    /// Whether this engine is ready to perform inference.
    fn is_ready(&self) -> bool;

    /// Initialize the engine (load model, connect to API, etc.).
    async fn initialize(&mut self) -> Result<()>;

    /// Perform inference and return the complete result.
    async fn infer(&self, prompt: &str, options: &InferenceOptions) -> Result<InferenceResult>;

    /// Perform streaming inference, calling the callback with each chunk.
    async fn infer_stream(
        &self,
        prompt: &str,
        options: &InferenceOptions,
        on_chunk: Box<dyn Fn(String) + Send + 'static>,
    ) -> Result<InferenceResult>;

    /// Shut down the engine and release resources.
    async fn shutdown(&mut self) -> Result<()>;
}
