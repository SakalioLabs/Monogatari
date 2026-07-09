//! Tests for the AI inference pipeline.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use async_trait::async_trait;
use llm_ai::{
    APIConfig, APIEngine, InferenceEngine, InferenceOptions, InferencePipeline, InferenceResult,
    ModelConfig, ONNXEngine, PromptBuilder,
};
use tokio::sync::RwLock;

struct UnsuccessfulResultEngine {
    infer_attempts: Arc<AtomicUsize>,
    stream_attempts: Arc<AtomicUsize>,
}

impl UnsuccessfulResultEngine {
    fn new(infer_attempts: Arc<AtomicUsize>, stream_attempts: Arc<AtomicUsize>) -> Self {
        Self {
            infer_attempts,
            stream_attempts,
        }
    }

    fn failed_result(&self) -> InferenceResult {
        InferenceResult {
            text: "must not be consumed".to_string(),
            success: false,
            error: Some("provider returned 429".to_string()),
            duration_ms: 1,
            tokens_generated: 0,
        }
    }
}

#[async_trait]
impl InferenceEngine for UnsuccessfulResultEngine {
    fn name(&self) -> &str {
        "FailingAPI"
    }

    fn is_ready(&self) -> bool {
        true
    }

    async fn initialize(&mut self) -> llm_core::Result<()> {
        Ok(())
    }

    async fn infer(
        &self,
        _prompt: &str,
        _options: &InferenceOptions,
    ) -> llm_core::Result<InferenceResult> {
        self.infer_attempts.fetch_add(1, Ordering::SeqCst);
        Ok(self.failed_result())
    }

    async fn infer_stream(
        &self,
        _prompt: &str,
        _options: &InferenceOptions,
        _on_chunk: Box<dyn Fn(String) + Send + 'static>,
    ) -> llm_core::Result<InferenceResult> {
        self.stream_attempts.fetch_add(1, Ordering::SeqCst);
        Ok(self.failed_result())
    }

    async fn shutdown(&mut self) -> llm_core::Result<()> {
        Ok(())
    }
}

#[test]
fn test_prompt_builder_empty() {
    let prompt = PromptBuilder::new().build();
    assert!(prompt.is_empty());
}

#[test]
fn test_prompt_builder_system_only() {
    let prompt = PromptBuilder::new()
        .system_prompt("You are a helpful assistant.")
        .build();
    assert!(prompt.contains("[System]"));
    assert!(prompt.contains("You are a helpful assistant."));
}

#[test]
fn test_prompt_builder_full_conversation() {
    let prompt = PromptBuilder::new()
        .system_prompt("You are a character in a game.")
        .character_context("Name: Sakura, Personality: Cheerful")
        .knowledge_context("Setting: Cherry blossom park")
        .user_message("Hello!")
        .assistant_message("Hi there! Nice to meet you!")
        .user_message("How are you?")
        .build();

    assert!(prompt.contains("[System]"));
    assert!(prompt.contains("Sakura"));
    assert!(prompt.contains("Cherry blossom"));
    assert!(prompt.contains("[User]"));
    assert!(prompt.contains("[Assistant]"));
}

#[test]
fn test_prompt_builder_world_context() {
    let prompt = PromptBuilder::new()
        .system_prompt("You are a game master.")
        .world_context("The world is set in a fantasy medieval era.")
        .build();

    assert!(prompt.contains("[System]"));
    assert!(prompt.contains("fantasy medieval"));
}

#[test]
fn test_inference_options_default() {
    let options = InferenceOptions::default();
    assert_eq!(options.max_tokens, 512);
    assert_eq!(options.temperature, 0.7);
    assert_eq!(options.top_p, 0.9);
    assert_eq!(options.top_k, 50);
    assert_eq!(options.repetition_penalty, 1.1);
    assert!(options.stop_sequences.is_empty());
}

#[test]
fn test_inference_pipeline_creation() {
    let pipeline = InferencePipeline::new();
    assert!(pipeline.engine_names().is_empty());
    assert!(pipeline.active_engine_name().is_none());
}

#[test]
fn test_inference_pipeline_no_active_engine() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let pipeline = InferencePipeline::new();
    let options = InferenceOptions::default();

    // Should return error when no active engine
    let result = rt.block_on(pipeline.generate_response("test", &options));
    assert!(result.is_err());
}

#[test]
fn test_inference_pipeline_register_engine_is_async_safe() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut pipeline = InferencePipeline::new();

    rt.block_on(async {
        pipeline.register_engine(Arc::new(RwLock::new(ONNXEngine::new(
            ModelConfig::default(),
        ))));
    });

    assert!(pipeline.engine_names().contains(&"ONNX"));
}

#[test]
fn test_inference_pipeline_engine_statuses_reflect_readiness() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut pipeline = InferencePipeline::new();
    let mut api_engine = APIEngine::new(APIConfig {
        base_url: "https://example.test/v1".to_string(),
        api_key: "test-key".to_string(),
        model: "test-model".to_string(),
        ..Default::default()
    });
    rt.block_on(api_engine.initialize()).unwrap();

    pipeline.register_engine(Arc::new(RwLock::new(api_engine)));
    pipeline.register_engine(Arc::new(RwLock::new(ONNXEngine::new(
        ModelConfig::default(),
    ))));

    let statuses = rt.block_on(pipeline.engine_statuses());

    assert_eq!(
        statuses,
        vec![("API".to_string(), true), ("ONNX".to_string(), false)]
    );
}

#[test]
fn test_inference_pipeline_retries_unsuccessful_results() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let infer_attempts = Arc::new(AtomicUsize::new(0));
    let stream_attempts = Arc::new(AtomicUsize::new(0));
    let mut pipeline = InferencePipeline::new();
    pipeline.register_engine(Arc::new(RwLock::new(UnsuccessfulResultEngine::new(
        Arc::clone(&infer_attempts),
        Arc::clone(&stream_attempts),
    ))));
    pipeline.set_active_engine("FailingAPI").unwrap();

    let error = rt
        .block_on(pipeline.generate_response("hello", &InferenceOptions::default()))
        .unwrap_err()
        .to_string();

    assert!(error.contains("FailingAPI"));
    assert!(error.contains("provider returned 429"));
    assert_eq!(infer_attempts.load(Ordering::SeqCst), 3);
    assert_eq!(stream_attempts.load(Ordering::SeqCst), 0);
}

#[test]
fn test_inference_pipeline_specific_engine_rejects_unsuccessful_result() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let infer_attempts = Arc::new(AtomicUsize::new(0));
    let stream_attempts = Arc::new(AtomicUsize::new(0));
    let mut pipeline = InferencePipeline::new();
    pipeline.register_engine(Arc::new(RwLock::new(UnsuccessfulResultEngine::new(
        Arc::clone(&infer_attempts),
        Arc::clone(&stream_attempts),
    ))));

    let error = rt
        .block_on(pipeline.generate_response_with_engine(
            "FailingAPI",
            "hello",
            &InferenceOptions::default(),
        ))
        .unwrap_err()
        .to_string();

    assert!(error.contains("provider returned 429"));
    assert_eq!(infer_attempts.load(Ordering::SeqCst), 1);
    assert_eq!(stream_attempts.load(Ordering::SeqCst), 0);
}

#[test]
fn test_inference_pipeline_stream_rejects_unsuccessful_result() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let infer_attempts = Arc::new(AtomicUsize::new(0));
    let stream_attempts = Arc::new(AtomicUsize::new(0));
    let emitted_chunks = Arc::new(AtomicUsize::new(0));
    let emitted_chunks_for_callback = Arc::clone(&emitted_chunks);
    let mut pipeline = InferencePipeline::new();
    pipeline.register_engine(Arc::new(RwLock::new(UnsuccessfulResultEngine::new(
        Arc::clone(&infer_attempts),
        Arc::clone(&stream_attempts),
    ))));
    pipeline.set_active_engine("FailingAPI").unwrap();

    let error = rt
        .block_on(pipeline.generate_stream(
            "hello",
            &InferenceOptions::default(),
            Box::new(move |_| {
                emitted_chunks_for_callback.fetch_add(1, Ordering::SeqCst);
            }),
        ))
        .unwrap_err()
        .to_string();

    assert!(error.contains("provider returned 429"));
    assert_eq!(infer_attempts.load(Ordering::SeqCst), 0);
    assert_eq!(stream_attempts.load(Ordering::SeqCst), 1);
    assert_eq!(emitted_chunks.load(Ordering::SeqCst), 0);
}

#[test]
fn test_prompt_builder_multiple_messages() {
    let prompt = PromptBuilder::new()
        .system_prompt("You are a helpful assistant.")
        .user_message("What is Rust?")
        .assistant_message("Rust is a systems programming language.")
        .user_message("Tell me more about its safety features.")
        .build();

    // Count user and assistant messages
    let user_count = prompt.matches("[User]").count();
    let assistant_count = prompt.matches("[Assistant]").count();
    assert_eq!(user_count, 2);
    assert_eq!(assistant_count, 1);
}

#[test]
fn test_prompt_builder_empty_messages() {
    let prompt = PromptBuilder::new()
        .system_prompt("System prompt")
        .user_message("")
        .assistant_message("")
        .build();

    // Empty messages should not be included
    assert!(!prompt.contains("[User]"));
    assert!(!prompt.contains("[Assistant]"));
}
