//! AI engine configuration and inference commands.

use serde::Serialize;
use tauri::State;

use llm_ai::{APIConfig, APIEngine, InferenceOptions, ModelConfig, ONNXEngine};

use crate::state::AppState;

#[derive(Serialize)]
pub struct AIStatus {
    pub engines: Vec<EngineInfo>,
    pub active_engine: Option<String>,
}

#[derive(Serialize)]
pub struct EngineInfo {
    pub name: String,
    pub ready: bool,
}

/// Configure the API inference engine.
#[tauri::command]
pub async fn configure_api(
    state: State<'_, AppState>,
    base_url: String,
    api_key: String,
    model: String,
) -> Result<String, String> {
    let config = APIConfig {
        base_url,
        api_key,
        model,
        ..Default::default()
    };

    let engine = APIEngine::new(config);
    let mut pipeline = state.inference_pipeline.write().await;
    pipeline.register_engine(std::sync::Arc::new(tokio::sync::RwLock::new(engine)));
    pipeline
        .set_active_engine("API")
        .map_err(|e| e.to_string())?;

    Ok("API engine configured".to_string())
}

/// Configure the ONNX inference engine.
#[tauri::command]
pub async fn configure_onnx(
    state: State<'_, AppState>,
    model_path: String,
    tokenizer_path: String,
) -> Result<String, String> {
    let config = ModelConfig {
        model_path: std::path::PathBuf::from(&model_path),
        tokenizer_path: std::path::PathBuf::from(&tokenizer_path),
        ..Default::default()
    };

    let engine = ONNXEngine::new(config);
    let mut pipeline = state.inference_pipeline.write().await;
    pipeline.register_engine(std::sync::Arc::new(tokio::sync::RwLock::new(engine)));

    Ok("ONNX engine configured".to_string())
}

/// Generate a response from the AI.
#[tauri::command]
pub async fn generate_response(
    state: State<'_, AppState>,
    prompt: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
) -> Result<String, String> {
    let options = InferenceOptions {
        max_tokens: max_tokens.unwrap_or(512),
        temperature: temperature.unwrap_or(0.7),
        ..Default::default()
    };

    let pipeline = state.inference_pipeline.read().await;
    let result = pipeline
        .generate_response(&prompt, &options)
        .await
        .map_err(|e| e.to_string())?;

    if result.success {
        Ok(result.text)
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

/// Get the current AI status.
#[tauri::command]
pub async fn get_ai_status(state: State<'_, AppState>) -> Result<AIStatus, String> {
    let pipeline = state.inference_pipeline.read().await;
    let engines = pipeline
        .engine_names()
        .into_iter()
        .map(|name| EngineInfo {
            name: name.to_string(),
            ready: true, // Would check actual engine readiness
        })
        .collect();

    Ok(AIStatus {
        engines,
        active_engine: pipeline.active_engine_name().map(|s| s.to_string()),
    })
}

/// Generate a streaming response, sending chunks via channel.
#[tauri::command]
pub async fn generate_stream(
    state: State<'_, AppState>,
    prompt: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    window: tauri::WebviewWindow,
) -> Result<String, String> {
    use tauri::Emitter;

    let options = InferenceOptions {
        max_tokens: max_tokens.unwrap_or(512),
        temperature: temperature.unwrap_or(0.7),
        ..Default::default()
    };

    let window_clone = window.clone();
    let on_chunk = Box::new(move |chunk: String| {
        let _ = window_clone.emit("llm-chunk", &chunk);
    });

    let pipeline = state.inference_pipeline.read().await;
    let result = pipeline
        .generate_stream(&prompt, &options, on_chunk)
        .await
        .map_err(|e| e.to_string())?;

    if result.success {
        let _ = window.emit("llm-complete", &result.text);
        Ok(result.text)
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}
