//! Tests for the AI inference pipeline.

use std::sync::Arc;

use llm_ai::{
    APIConfig, APIEngine, InferenceEngine, InferenceOptions, InferencePipeline, ModelConfig,
    ONNXEngine, PromptBuilder,
};
use tokio::sync::RwLock;

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
