//! # LLM Galgame Engine - AI
//!
//! LLM inference layer supporting both API-based and local ONNX models.
//!
//! ## Architecture
//!
//! The AI module follows a strategy pattern with multiple inference engines:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    InferencePipeline                        │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
//! │  │  API Engine  │  │ ONNX Engine │  │  Engine N   │        │
//! │  │ (OpenAI等)   │  │ (本地模型)   │  │  (扩展)     │        │
//! │  └─────────────┘  └─────────────┘  └─────────────┘        │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//!                     InferenceResult
//! ```
//!
//! ## Supported Engines
//!
//! ### API Engine
//! - OpenAI-compatible API (GPT-3.5, GPT-4, etc.)
//! - Custom API endpoints
//! - Streaming support via SSE
//!
//! ### ONNX Engine
//! - Local ONNX model inference
//! - DirectML GPU acceleration (Windows)
//! - CPU fallback
//! - Temperature, top-k, top-p sampling
//!
//! ## Usage
//!
//! ```rust,no_run
//! use llm_ai::{InferencePipeline, APIEngine, APIConfig, InferenceOptions};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create and configure API engine
//! let config = APIConfig {
//!     base_url: "https://api.openai.com/v1".to_string(),
//!     api_key: "your-key".to_string(),
//!     model: "gpt-3.5-turbo".to_string(),
//!     ..Default::default()
//! };
//! let engine = APIEngine::new(config);
//!
//! // Create pipeline and register engine
//! let mut pipeline = InferencePipeline::new();
//! pipeline.register_engine(std::sync::Arc::new(tokio::sync::RwLock::new(engine)));
//! pipeline.set_active_engine("API")?;
//!
//! // Generate response
//! let options = InferenceOptions::default();
//! let result = pipeline.generate_response("Hello!", &options).await?;
//! println!("Response: {}", result.text);
//! # Ok(())
//! # }
//! ```

pub mod api_engine;
pub mod inference;
pub mod onnx_engine;
pub mod pipeline;
pub mod prompt_builder;

pub use api_engine::{APIConfig, APIEngine};
pub use inference::{InferenceEngine, InferenceOptions, InferenceResult};
pub use onnx_engine::{ModelConfig, ONNXEngine};
pub use pipeline::InferencePipeline;
pub use prompt_builder::PromptBuilder;
