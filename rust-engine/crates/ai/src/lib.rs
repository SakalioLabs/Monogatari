//! # LLM Galgame Engine - AI
//!
//! LLM inference layer with conservative backend planning, API services, and a
//! linked DirectML executor for compatible local ONNX models.
//!
//! ## Architecture
//!
//! ```text
//! Host detection + completed probes
//!                 |
//!                 v
//!       Inference backend plan
//!                 |
//!       +---------+---------+
//!       |                   |
//!       v                   v
//! API/service engine   DirectML ONNX engine
//!       |                   |
//!       +---------+---------+
//!                 v
//!          InferenceResult
//! ```
//!
//! ## Supported Engines
//!
//! ### API Engine
//! - OpenAI-compatible APIs and local services
//! - Custom API endpoints
//! - Streaming support via SSE
//!
//! ### ONNX Engine
//! - Compatible full-sequence local ONNX model inference
//! - Required DirectML GPU acceleration on Windows
//! - No implicit CPU fallback
//! - Temperature, top-k, and top-p sampling
//! - Qwen3.5 hybrid recurrent-state graphs are not supported by this executor
//!
//! ## Usage
//!
//! ```rust,no_run
//! use llm_ai::{APIConfig, APIEngine, InferenceOptions, InferencePipeline};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = APIConfig {
//!     base_url: "https://api.openai.com/v1".to_string(),
//!     api_key: "your-key".to_string(),
//!     model: "gpt-4o-mini".to_string(),
//!     ..Default::default()
//! };
//! let engine = APIEngine::new(config);
//!
//! let mut pipeline = InferencePipeline::new();
//! pipeline.register_engine(std::sync::Arc::new(tokio::sync::RwLock::new(engine)));
//! pipeline.set_active_engine("API")?;
//!
//! let result = pipeline
//!     .generate_response("Hello!", &InferenceOptions::default())
//!     .await?;
//! println!("Response: {}", result.text);
//! # Ok(())
//! # }
//! ```

pub mod api_engine;
pub mod backend_selection;
pub mod inference;
pub mod onnx_engine;
pub mod pipeline;
pub mod prompt_builder;

pub use api_engine::{APIConfig, APIEngine};
pub use backend_selection::{
    build_inference_backend_plan, detect_host_capabilities, AccelerationKind, BackendAssessment,
    BackendPlanRequest, BackendReadiness, DeploymentTarget, HostCapabilities, InferenceBackendId,
    InferenceBackendPlan, ModelProfile, RuntimeProbeSignals, INFERENCE_BACKEND_PLAN_SCHEMA,
};
pub use inference::{InferenceEngine, InferenceOptions, InferenceResult};
pub use onnx_engine::{ModelConfig, ONNXEngine};
pub use pipeline::InferencePipeline;
pub use prompt_builder::PromptBuilder;
