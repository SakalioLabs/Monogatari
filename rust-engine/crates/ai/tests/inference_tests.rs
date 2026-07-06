//! Tests for AI inference options and results.

use llm_ai::{InferenceOptions, InferenceResult};

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
fn test_inference_options_clone() {
    let options = InferenceOptions {
        max_tokens: 1024,
        temperature: 0.5,
        top_p: 0.8,
        top_k: 40,
        repetition_penalty: 1.2,
        stop_sequences: vec!["\n".to_string()],
    };
    
    let cloned = options.clone();
    assert_eq!(cloned.max_tokens, 1024);
    assert_eq!(cloned.temperature, 0.5);
    assert_eq!(cloned.top_p, 0.8);
    assert_eq!(cloned.top_k, 40);
    assert_eq!(cloned.repetition_penalty, 1.2);
    assert_eq!(cloned.stop_sequences, vec!["\n".to_string()]);
}

#[test]
fn test_inference_options_serialization() {
    let options = InferenceOptions {
        max_tokens: 256,
        temperature: 0.8,
        top_p: 0.95,
        top_k: 30,
        repetition_penalty: 1.0,
        stop_sequences: vec!["END".to_string()],
    };
    
    let json = serde_json::to_string(&options).unwrap();
    assert!(json.contains("256"));
    assert!(json.contains("0.8"));
    
    let deserialized: InferenceOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.max_tokens, 256);
    assert_eq!(deserialized.temperature, 0.8);
}

#[test]
fn test_inference_result_success() {
    let result = InferenceResult {
        text: "Hello, world!".to_string(),
        success: true,
        error: None,
        duration_ms: 100,
        tokens_generated: 5,
    };
    
    assert!(result.success);
    assert_eq!(result.text, "Hello, world!");
    assert!(result.error.is_none());
    assert_eq!(result.duration_ms, 100);
    assert_eq!(result.tokens_generated, 5);
}

#[test]
fn test_inference_result_failure() {
    let result = InferenceResult {
        text: String::new(),
        success: false,
        error: Some("Connection timeout".to_string()),
        duration_ms: 5000,
        tokens_generated: 0,
    };
    
    assert!(!result.success);
    assert!(result.text.is_empty());
    assert_eq!(result.error, Some("Connection timeout".to_string()));
}

#[test]
fn test_inference_result_clone() {
    let result = InferenceResult {
        text: "Test response".to_string(),
        success: true,
        error: None,
        duration_ms: 50,
        tokens_generated: 3,
    };
    
    let cloned = result.clone();
    assert_eq!(cloned.text, "Test response");
    assert!(cloned.success);
}

#[test]
fn test_inference_result_serialization() {
    let result = InferenceResult {
        text: "Serialized".to_string(),
        success: true,
        error: None,
        duration_ms: 75,
        tokens_generated: 2,
    };
    
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("Serialized"));
    assert!(json.contains("true"));
    
    let deserialized: InferenceResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.text, "Serialized");
    assert!(deserialized.success);
}

#[test]
fn test_inference_options_temperature_bounds() {
    // Test low temperature (more deterministic)
    let options = InferenceOptions {
        temperature: 0.1,
        ..Default::default()
    };
    assert!(options.temperature < 0.5);
    
    // Test high temperature (more random)
    let options = InferenceOptions {
        temperature: 1.5,
        ..Default::default()
    };
    assert!(options.temperature > 1.0);
}

#[test]
fn test_inference_options_max_tokens() {
    let options = InferenceOptions {
        max_tokens: 1,
        ..Default::default()
    };
    assert_eq!(options.max_tokens, 1);
    
    let options = InferenceOptions {
        max_tokens: 4096,
        ..Default::default()
    };
    assert_eq!(options.max_tokens, 4096);
}

#[test]
fn test_inference_options_stop_sequences() {
    let options = InferenceOptions {
        stop_sequences: vec![
            "\n\n".to_string(),
            "END".to_string(),
            "STOP".to_string(),
        ],
        ..Default::default()
    };
    
    assert_eq!(options.stop_sequences.len(), 3);
    assert!(options.stop_sequences.contains(&"\n\n".to_string()));
}

#[test]
fn test_inference_result_duration() {
    let result = InferenceResult {
        text: "Fast".to_string(),
        success: true,
        error: None,
        duration_ms: 10,
        tokens_generated: 1,
    };
    
    assert!(result.duration_ms < 100);
    
    let result = InferenceResult {
        text: "Slow".to_string(),
        success: true,
        error: None,
        duration_ms: 10000,
        tokens_generated: 100,
    };
    
    assert!(result.duration_ms > 1000);
}
