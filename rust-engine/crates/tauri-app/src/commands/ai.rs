//! AI engine configuration and inference commands.

use serde::Serialize;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

use llm_ai::{APIConfig, APIEngine, InferenceOptions, InferencePipeline, ModelConfig, ONNXEngine};

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

fn onnx_model_config_in_project(
    project_root: &Path,
    model_path: &str,
    tokenizer_path: &str,
) -> Result<ModelConfig, String> {
    Ok(ModelConfig {
        model_path: onnx_file_path_in_project(project_root, model_path, &[".onnx"], "ONNX model")?,
        tokenizer_path: onnx_file_path_in_project(
            project_root,
            tokenizer_path,
            &[".json"],
            "ONNX tokenizer",
        )?,
        ..Default::default()
    })
}

fn onnx_file_path_in_project(
    project_root: &Path,
    file_ref: &str,
    allowed_extensions: &[&str],
    label: &str,
) -> Result<PathBuf, String> {
    let segments = normalize_onnx_file_ref(file_ref)?;
    let relative = segments.join("/");
    let lower = relative.to_ascii_lowercase();
    if !allowed_extensions
        .iter()
        .any(|extension| lower.ends_with(extension))
    {
        return Err(format!(
            "{label} paths must point to a {} file.",
            allowed_extensions.join(" or ")
        ));
    }

    let path = segments
        .iter()
        .fold(project_root.to_path_buf(), |path, segment| {
            path.join(segment)
        });
    if !path.starts_with(project_root) {
        return Err("ONNX file paths must stay inside active project data root.".to_string());
    }

    Ok(path)
}

fn normalize_onnx_file_ref(file_ref: &str) -> Result<Vec<String>, String> {
    let normalized = file_ref.trim().replace('\\', "/");
    if normalized.is_empty() || normalized.chars().any(char::is_control) {
        return Err(
            "ONNX file paths must be non-empty and cannot contain control characters.".to_string(),
        );
    }
    if normalized.contains(':') {
        return Err("ONNX file paths cannot contain drive prefixes or URI schemes.".to_string());
    }

    let segments = normalized.split('/').collect::<Vec<_>>();
    if segments
        .iter()
        .any(|segment| segment.is_empty() || *segment == "." || *segment == "..")
    {
        return Err(
            "ONNX file paths cannot contain empty, current, or parent directory segments."
                .to_string(),
        );
    }
    if segments.iter().any(|segment| {
        !segment
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    }) {
        return Err(
            "ONNX file paths can contain only ASCII letters, numbers, underscores, hyphens, dots, or separators."
                .to_string(),
        );
    }

    let path = Path::new(&normalized);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return Err(
            "ONNX file paths must be relative to the active project data root.".to_string(),
        );
    }

    Ok(segments.into_iter().map(str::to_string).collect())
}

fn register_onnx_engine(
    pipeline: &mut InferencePipeline,
    config: ModelConfig,
) -> Result<(), String> {
    let engine = ONNXEngine::new(config);
    pipeline.register_engine(Arc::new(RwLock::new(engine)));
    pipeline
        .set_active_engine("ONNX")
        .map_err(|e| e.to_string())?;
    Ok(())
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
    let project_root = state.current_project_data_root().await;
    let config = onnx_model_config_in_project(&project_root, &model_path, &tokenizer_path)?;
    let mut pipeline = state.inference_pipeline.write().await;
    register_onnx_engine(&mut pipeline, config)?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn onnx_file_paths_resolve_under_project_root() {
        let root = PathBuf::from("project-data");
        let config =
            onnx_model_config_in_project(&root, "models/model.onnx", "models/tokenizer.json")
                .unwrap();

        assert_eq!(config.model_path, root.join("models").join("model.onnx"));
        assert_eq!(
            config.tokenizer_path,
            root.join("models").join("tokenizer.json")
        );
        assert_eq!(
            onnx_file_path_in_project(&root, "models\\qwen\\model.onnx", &[".onnx"], "ONNX model")
                .unwrap(),
            root.join("models").join("qwen").join("model.onnx")
        );
    }

    #[test]
    fn onnx_file_paths_reject_escape_attempts() {
        let root = PathBuf::from("project-data");
        for file_ref in [
            "",
            "../model.onnx",
            "models/../model.onnx",
            "models//model.onnx",
            "models/./model.onnx",
            "C:/Users/example/model.onnx",
            "https://example.test/model.onnx",
            "/tmp/model.onnx",
            "models/model.bin",
            "models/model path.onnx",
            "models/model!.onnx",
        ] {
            assert!(
                onnx_file_path_in_project(&root, file_ref, &[".onnx"], "ONNX model").is_err(),
                "{file_ref} should be rejected"
            );
        }
        assert!(onnx_file_path_in_project(
            &root,
            "models/tokenizer.txt",
            &[".json"],
            "ONNX tokenizer"
        )
        .is_err());
    }

    #[test]
    fn configure_onnx_registers_active_engine() {
        let root = PathBuf::from("project-data");
        let config =
            onnx_model_config_in_project(&root, "models/model.onnx", "models/tokenizer.json")
                .unwrap();
        let mut pipeline = InferencePipeline::new();

        register_onnx_engine(&mut pipeline, config).unwrap();

        assert_eq!(pipeline.active_engine_name(), Some("ONNX"));
        assert!(pipeline.engine_names().contains(&"ONNX"));
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
