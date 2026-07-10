//! ONNX Runtime inference engine for Windows DirectML execution.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::info;

use llm_core::Result;

use crate::inference::{InferenceEngine, InferenceOptions, InferenceResult};

#[cfg(target_os = "windows")]
use ort::{ep, session::Session, value::Tensor};
#[cfg(target_os = "windows")]
use rand::Rng;
#[cfg(target_os = "windows")]
use std::collections::HashSet;
#[cfg(target_os = "windows")]
use std::sync::Mutex;
#[cfg(target_os = "windows")]
use tokenizers::Tokenizer;

pub const ONNX_RUNTIME_UNAVAILABLE_MESSAGE: &str =
    "DirectML inference is available only in Windows builds.";

/// Configuration for the Windows ONNX inference engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_path: PathBuf,
    pub tokenizer_path: PathBuf,
    pub max_sequence_length: usize,
    pub vocab_size: usize,
    pub use_directml: bool,
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

#[cfg(target_os = "windows")]
struct DirectMLRuntime {
    session: Mutex<Session>,
    tokenizer: Tokenizer,
    input_names: HashSet<String>,
    eos_token_ids: HashSet<u32>,
    max_sequence_length: usize,
}

/// Local LLM executor used by Windows packages.
pub struct ONNXEngine {
    config: ModelConfig,
    #[cfg(target_os = "windows")]
    runtime: Option<Arc<DirectMLRuntime>>,
}

impl ONNXEngine {
    pub fn new(mut config: ModelConfig) -> Self {
        // Windows packages have one local execution contract: DirectML.
        config.use_directml = true;
        Self {
            config,
            #[cfg(target_os = "windows")]
            runtime: None,
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn runtime_unavailable_error() -> llm_core::EngineError {
        llm_core::EngineError::inference("DirectML", ONNX_RUNTIME_UNAVAILABLE_MESSAGE)
    }

    #[cfg(target_os = "windows")]
    fn not_initialized_error() -> llm_core::EngineError {
        llm_core::EngineError::inference(
            "DirectML",
            "DirectML model is not initialized. Configure a compatible ONNX model first.",
        )
    }

    fn validate_files(&self) -> Result<()> {
        if !self.config.model_path.is_file() {
            return Err(llm_core::EngineError::config(
                "model_path",
                format!("Model file not found: {}", self.config.model_path.display()),
            ));
        }
        if !self.config.tokenizer_path.is_file() {
            return Err(llm_core::EngineError::config(
                "tokenizer_path",
                format!(
                    "Tokenizer file not found: {}",
                    self.config.tokenizer_path.display()
                ),
            ));
        }
        if self.config.device_id < 0 {
            return Err(llm_core::EngineError::config(
                "device_id",
                "DirectML device ID cannot be negative.",
            ));
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
impl DirectMLRuntime {
    fn load(config: &ModelConfig) -> Result<Self> {
        let directml = ep::DirectML::default()
            .with_device_id(config.device_id)
            .build()
            .error_on_failure();
        let session = Session::builder()
            .map_err(|error| directml_error("create session builder", error))?
            .with_execution_providers([directml])
            .map_err(|error| directml_error("register DirectML", error))?
            .with_parallel_execution(false)
            .map_err(|error| directml_error("select sequential execution", error))?
            .with_memory_pattern(false)
            .map_err(|error| directml_error("disable memory patterns", error))?
            .commit_from_file(&config.model_path)
            .map_err(|error| directml_error("load ONNX model", error))?;

        let input_names = session
            .inputs()
            .iter()
            .map(|input| input.name().to_string())
            .collect::<HashSet<_>>();
        validate_model_contract(&session, &input_names)?;

        let tokenizer = Tokenizer::from_file(&config.tokenizer_path).map_err(|error| {
            llm_core::EngineError::config(
                "tokenizer_path",
                format!("Failed to load tokenizer.json: {error}"),
            )
        })?;
        let eos_token_ids = ["</s>", "<|endoftext|>", "<|im_end|>", "<|eot_id|>"]
            .into_iter()
            .filter_map(|token| tokenizer.token_to_id(token))
            .collect();

        Ok(Self {
            session: Mutex::new(session),
            tokenizer,
            input_names,
            eos_token_ids,
            max_sequence_length: config.max_sequence_length.max(2),
        })
    }

    fn generate(
        &self,
        prompt: &str,
        options: &InferenceOptions,
        on_chunk: Option<Box<dyn Fn(String) + Send + 'static>>,
    ) -> Result<InferenceResult> {
        let started = Instant::now();
        let encoding = self.tokenizer.encode(prompt, true).map_err(|error| {
            llm_core::EngineError::inference("DirectML", format!("Tokenization failed: {error}"))
        })?;
        let mut token_ids = encoding.get_ids().to_vec();
        if token_ids.is_empty() {
            return Err(llm_core::EngineError::inference(
                "DirectML",
                "The prompt produced no input tokens.",
            ));
        }
        if token_ids.len() >= self.max_sequence_length {
            let keep = self.max_sequence_length - 1;
            token_ids = token_ids[token_ids.len() - keep..].to_vec();
        }

        let mut generated = Vec::<u32>::new();
        let mut emitted_text = String::new();

        for _ in 0..options.max_tokens {
            if token_ids.len() >= self.max_sequence_length {
                break;
            }
            let logits = self.next_token_logits(&token_ids)?;
            let next = sample_token(&logits, &token_ids, options)?;
            if self.eos_token_ids.contains(&next) {
                break;
            }
            token_ids.push(next);
            generated.push(next);

            let decoded = self.decode(&generated)?;
            let visible = text_before_stop(&decoded, &options.stop_sequences);
            if let Some(callback) = on_chunk.as_ref() {
                if let Some(delta) = visible.strip_prefix(&emitted_text) {
                    if !delta.is_empty() {
                        callback(delta.to_string());
                        emitted_text.push_str(delta);
                    }
                }
            }
            if visible.len() < decoded.len() {
                break;
            }
        }

        let decoded = self.decode(&generated)?;
        let text = text_before_stop(&decoded, &options.stop_sequences).to_string();
        if let Some(callback) = on_chunk.as_ref() {
            if let Some(delta) = text.strip_prefix(&emitted_text) {
                if !delta.is_empty() {
                    callback(delta.to_string());
                }
            }
        }

        Ok(InferenceResult {
            text,
            success: true,
            error: None,
            duration_ms: started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
            tokens_generated: generated.len() as u32,
        })
    }

    fn next_token_logits(&self, token_ids: &[u32]) -> Result<Vec<f32>> {
        let shape = [1_usize, token_ids.len()];
        let input_ids = token_ids
            .iter()
            .map(|token| i64::from(*token))
            .collect::<Vec<_>>();
        let mut inputs = ort::inputs![
            "input_ids" => Tensor::from_array((shape, input_ids))
                .map_err(|error| directml_error("create input_ids tensor", error))?
        ];

        if self.input_names.contains("attention_mask") {
            inputs.push((
                "attention_mask".into(),
                Tensor::from_array((shape, vec![1_i64; token_ids.len()]))
                    .map_err(|error| directml_error("create attention_mask tensor", error))?
                    .into(),
            ));
        }
        if self.input_names.contains("position_ids") {
            inputs.push((
                "position_ids".into(),
                Tensor::from_array((
                    shape,
                    (0..token_ids.len())
                        .map(|position| position as i64)
                        .collect::<Vec<_>>(),
                ))
                .map_err(|error| directml_error("create position_ids tensor", error))?
                .into(),
            ));
        }
        if self.input_names.contains("token_type_ids") {
            inputs.push((
                "token_type_ids".into(),
                Tensor::from_array((shape, vec![0_i64; token_ids.len()]))
                    .map_err(|error| directml_error("create token_type_ids tensor", error))?
                    .into(),
            ));
        }

        let mut session = self.session.lock().map_err(|_| {
            llm_core::EngineError::inference("DirectML", "The inference session lock is poisoned.")
        })?;
        let outputs = session
            .run(inputs)
            .map_err(|error| directml_error("execute model", error))?;
        let logits = outputs.get("logits").ok_or_else(|| {
            llm_core::EngineError::inference("DirectML", "The model returned no logits output.")
        })?;
        let (output_shape, values) = logits
            .try_extract_tensor::<f32>()
            .map_err(|error| directml_error("read logits output", error))?;
        let vocab_size = output_shape
            .iter()
            .last()
            .copied()
            .unwrap_or_default()
            .max(0) as usize;
        if vocab_size == 0 || values.len() < vocab_size {
            return Err(llm_core::EngineError::inference(
                "DirectML",
                "The logits output has an invalid vocabulary dimension.",
            ));
        }
        Ok(values[values.len() - vocab_size..].to_vec())
    }

    fn decode(&self, token_ids: &[u32]) -> Result<String> {
        self.tokenizer.decode(token_ids, true).map_err(|error| {
            llm_core::EngineError::inference("DirectML", format!("Token decoding failed: {error}"))
        })
    }
}

#[cfg(target_os = "windows")]
fn validate_model_contract(session: &Session, input_names: &HashSet<String>) -> Result<()> {
    if !input_names.contains("input_ids") {
        return Err(llm_core::EngineError::config(
            "model_path",
            "The ONNX model must expose an input_ids input.",
        ));
    }
    let supported = [
        "input_ids",
        "attention_mask",
        "position_ids",
        "token_type_ids",
    ]
    .into_iter()
    .collect::<HashSet<_>>();
    let unsupported = input_names
        .iter()
        .filter(|name| !supported.contains(name.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !unsupported.is_empty() {
        return Err(llm_core::EngineError::config(
            "model_path",
            format!(
                "This model requires unsupported inputs: {}. Export a full-sequence causal LM graph without external KV-cache inputs.",
                unsupported.join(", ")
            ),
        ));
    }
    if !session
        .outputs()
        .iter()
        .any(|output| output.name() == "logits")
    {
        return Err(llm_core::EngineError::config(
            "model_path",
            "The ONNX model must expose a float32 logits output.",
        ));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn directml_error(action: &str, error: impl std::fmt::Display) -> llm_core::EngineError {
    llm_core::EngineError::inference("DirectML", format!("Failed to {action}: {error}"))
}

#[cfg(target_os = "windows")]
fn sample_token(logits: &[f32], generated: &[u32], options: &InferenceOptions) -> Result<u32> {
    if logits.is_empty() {
        return Err(llm_core::EngineError::inference(
            "DirectML",
            "The model returned an empty logits vector.",
        ));
    }

    let penalty = options.repetition_penalty.max(1.0);
    let mut candidates = logits
        .iter()
        .enumerate()
        .filter_map(|(token, value)| {
            if !value.is_finite() {
                return None;
            }
            let adjusted = if generated.contains(&(token as u32)) {
                if *value >= 0.0 {
                    *value / penalty
                } else {
                    *value * penalty
                }
            } else {
                *value
            };
            Some((token as u32, adjusted))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| right.1.total_cmp(&left.1));
    if candidates.is_empty() {
        return Err(llm_core::EngineError::inference(
            "DirectML",
            "The model returned no finite token logits.",
        ));
    }

    if options.temperature <= f32::EPSILON {
        return Ok(candidates[0].0);
    }
    let top_k = if options.top_k == 0 {
        candidates.len()
    } else {
        (options.top_k as usize).min(candidates.len()).max(1)
    };
    candidates.truncate(top_k);

    let max_logit = candidates[0].1;
    let temperature = options.temperature.clamp(0.01, 2.0);
    let mut weighted = candidates
        .into_iter()
        .map(|(token, logit)| (token, ((logit - max_logit) / temperature).exp()))
        .collect::<Vec<_>>();
    let total = weighted.iter().map(|(_, weight)| *weight).sum::<f32>();
    if !total.is_finite() || total <= 0.0 {
        return Ok(weighted[0].0);
    }

    let top_p = options.top_p.clamp(0.01, 1.0);
    let mut cumulative = 0.0;
    let mut keep = weighted.len();
    for (index, (_, weight)) in weighted.iter().enumerate() {
        cumulative += *weight / total;
        if cumulative >= top_p {
            keep = index + 1;
            break;
        }
    }
    weighted.truncate(keep.max(1));
    let filtered_total = weighted.iter().map(|(_, weight)| *weight).sum::<f32>();
    let mut sample = rand::thread_rng().gen_range(0.0..filtered_total);
    for (token, weight) in &weighted {
        if sample <= *weight {
            return Ok(*token);
        }
        sample -= *weight;
    }
    Ok(weighted.last().map(|entry| entry.0).unwrap_or_default())
}

fn text_before_stop<'a>(text: &'a str, stops: &[String]) -> &'a str {
    let stop_index = stops
        .iter()
        .filter(|stop| !stop.is_empty())
        .filter_map(|stop| text.find(stop))
        .min()
        .unwrap_or(text.len());
    &text[..stop_index]
}

#[async_trait]
impl InferenceEngine for ONNXEngine {
    fn name(&self) -> &str {
        "ONNX"
    }

    fn is_ready(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            self.runtime.is_some()
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }

    async fn initialize(&mut self) -> Result<()> {
        self.validate_files()?;
        info!(
            "Initializing Windows DirectML engine with model: {}",
            self.config.model_path.display()
        );

        #[cfg(target_os = "windows")]
        {
            let config = self.config.clone();
            let runtime = tokio::task::spawn_blocking(move || DirectMLRuntime::load(&config))
                .await
                .map_err(|error| {
                    llm_core::EngineError::inference(
                        "DirectML",
                        format!("Model initialization task failed: {error}"),
                    )
                })??;
            self.runtime = Some(Arc::new(runtime));
            info!("Windows DirectML engine initialized");
            return Ok(());
        }

        #[cfg(not(target_os = "windows"))]
        Err(Self::runtime_unavailable_error())
    }

    async fn infer(&self, prompt: &str, options: &InferenceOptions) -> Result<InferenceResult> {
        #[cfg(target_os = "windows")]
        {
            let runtime = self
                .runtime
                .as_ref()
                .cloned()
                .ok_or_else(Self::not_initialized_error)?;
            let prompt = prompt.to_string();
            let options = options.clone();
            return tokio::task::spawn_blocking(move || runtime.generate(&prompt, &options, None))
                .await
                .map_err(|error| {
                    llm_core::EngineError::inference(
                        "DirectML",
                        format!("Inference task failed: {error}"),
                    )
                })?;
        }

        #[cfg(not(target_os = "windows"))]
        Err(Self::runtime_unavailable_error())
    }

    async fn infer_stream(
        &self,
        prompt: &str,
        options: &InferenceOptions,
        on_chunk: Box<dyn Fn(String) + Send + 'static>,
    ) -> Result<InferenceResult> {
        #[cfg(target_os = "windows")]
        {
            let runtime = self
                .runtime
                .as_ref()
                .cloned()
                .ok_or_else(Self::not_initialized_error)?;
            let prompt = prompt.to_string();
            let options = options.clone();
            return tokio::task::spawn_blocking(move || {
                runtime.generate(&prompt, &options, Some(on_chunk))
            })
            .await
            .map_err(|error| {
                llm_core::EngineError::inference(
                    "DirectML",
                    format!("Streaming task failed: {error}"),
                )
            })?;
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = (prompt, options, on_chunk);
            Err(Self::runtime_unavailable_error())
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            self.runtime = None;
        }
        info!("DirectML engine shut down");
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
        std::fs::write(dir.join("model.onnx"), b"invalid model fixture").unwrap();
        std::fs::write(dir.join("tokenizer.json"), b"{}").unwrap();
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
    fn onnx_engine_enforces_directml_contract() {
        let dir = temp_model_dir("contract");
        let mut config = test_config(&dir);
        config.use_directml = false;
        let engine = ONNXEngine::new(config);

        assert!(engine.config.use_directml);
        assert!(!engine.is_ready());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn onnx_initialize_rejects_invalid_model_without_ready_state() {
        let dir = temp_model_dir("initialize");
        let mut engine = ONNXEngine::new(test_config(&dir));
        let rt = tokio::runtime::Runtime::new().unwrap();

        let error = rt.block_on(engine.initialize()).unwrap_err();

        #[cfg(target_os = "windows")]
        assert!(error.to_string().contains("load ONNX model"));
        #[cfg(not(target_os = "windows"))]
        assert!(error.to_string().contains(ONNX_RUNTIME_UNAVAILABLE_MESSAGE));
        assert!(!engine.is_ready());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn onnx_infer_requires_initialized_runtime() {
        let dir = temp_model_dir("infer");
        let engine = ONNXEngine::new(test_config(&dir));
        let rt = tokio::runtime::Runtime::new().unwrap();

        let error = rt
            .block_on(engine.infer("hello", &InferenceOptions::default()))
            .unwrap_err();

        #[cfg(target_os = "windows")]
        assert!(error.to_string().contains("not initialized"));
        #[cfg(not(target_os = "windows"))]
        assert!(error.to_string().contains(ONNX_RUNTIME_UNAVAILABLE_MESSAGE));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn stop_sequences_are_removed_from_generated_text() {
        let stops = vec!["<stop>".to_string()];
        assert_eq!(text_before_stop("hello<stop>ignored", &stops), "hello");
        assert_eq!(text_before_stop("hello", &stops), "hello");
    }
}
