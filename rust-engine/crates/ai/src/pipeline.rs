//! Inference pipeline that manages multiple engines and routes requests.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{info, warn};

use llm_core::Result;

use crate::inference::{InferenceEngine, InferenceOptions, InferenceResult};

/// Manages multiple inference engines and routes requests.
pub struct InferencePipeline {
    engines: HashMap<String, Arc<RwLock<dyn InferenceEngine>>>,
    active_engine: Option<String>,
}

impl InferencePipeline {
    /// Create a new empty pipeline.
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
            active_engine: None,
        }
    }

    /// Register an inference engine.
    pub fn register_engine(&mut self, engine: Arc<RwLock<dyn InferenceEngine>>) {
        let name = {
            let engine = engine
                .try_read()
                .expect("register_engine requires an unlocked engine");
            engine.name().to_string()
        };
        self.register_engine_with_name(name, engine);
    }

    /// Register an inference engine under an explicit name.
    pub fn register_engine_with_name(
        &mut self,
        name: impl Into<String>,
        engine: Arc<RwLock<dyn InferenceEngine>>,
    ) {
        let name = name.into();
        info!("Registered inference engine: {}", name);
        self.engines.insert(name, engine);
    }

    /// Set the active engine by name.
    pub fn set_active_engine(&mut self, name: &str) -> Result<()> {
        if self.engines.contains_key(name) {
            self.active_engine = Some(name.to_string());
            info!("Active inference engine set to: {}", name);
            Ok(())
        } else {
            Err(llm_core::EngineError::inference(name, "Engine not found"))
        }
    }

    /// Get the name of the active engine.
    pub fn active_engine_name(&self) -> Option<&str> {
        self.active_engine.as_deref()
    }

    /// List all registered engine names.
    pub fn engine_names(&self) -> Vec<&str> {
        self.engines.keys().map(|s| s.as_str()).collect()
    }

    /// List registered engines with their actual readiness state.
    pub async fn engine_statuses(&self) -> Vec<(String, bool)> {
        let mut names = self.engines.keys().cloned().collect::<Vec<_>>();
        names.sort();

        let mut statuses = Vec::with_capacity(names.len());
        for name in names {
            if let Some(engine) = self.engines.get(&name) {
                let engine = engine.read().await;
                statuses.push((name, engine.is_ready()));
            }
        }

        statuses
    }

    /// Initialize all registered engines.
    pub async fn initialize_all(&self) -> Result<()> {
        for (name, engine) in &self.engines {
            info!("Initializing engine: {}", name);
            let mut engine = engine.write().await;
            if let Err(e) = engine.initialize().await {
                warn!("Failed to initialize engine {}: {}", name, e);
            }
        }
        Ok(())
    }

    /// Generate a response using the active engine with a pre-built prompt.
    /// Includes retry logic for transient failures.
    pub async fn generate_response(
        &self,
        prompt: &str,
        options: &InferenceOptions,
    ) -> Result<InferenceResult> {
        let engine_name = self.active_engine.as_ref().ok_or_else(|| {
            llm_core::EngineError::inference("none", "No active inference engine")
        })?;

        let engine = self
            .engines
            .get(engine_name)
            .ok_or_else(|| llm_core::EngineError::inference(engine_name, "Engine not found"))?;

        let engine = engine.read().await;

        // Try with retry logic
        let max_retries = 3;
        let mut last_error = None;

        for attempt in 1..=max_retries {
            match engine.infer(prompt, options).await {
                Ok(result) => {
                    if attempt > 1 {
                        info!("Inference succeeded on attempt {}", attempt);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    warn!("Inference attempt {} failed: {}", attempt, e);
                    last_error = Some(e);
                    if attempt < max_retries {
                        // Exponential backoff
                        tokio::time::sleep(std::time::Duration::from_millis(100 * attempt as u64))
                            .await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            llm_core::EngineError::inference("pipeline", "All retry attempts failed")
        }))
    }

    /// Generate a response using a specific engine by name.
    pub async fn generate_response_with_engine(
        &self,
        engine_name: &str,
        prompt: &str,
        options: &InferenceOptions,
    ) -> Result<InferenceResult> {
        let engine = self
            .engines
            .get(engine_name)
            .ok_or_else(|| llm_core::EngineError::inference(engine_name, "Engine not found"))?;

        let engine = engine.read().await;
        engine.infer(prompt, options).await
    }

    /// Generate a streaming response using the active engine.
    pub async fn generate_stream(
        &self,
        prompt: &str,
        options: &InferenceOptions,
        on_chunk: Box<dyn Fn(String) + Send + 'static>,
    ) -> Result<InferenceResult> {
        let engine_name = self.active_engine.as_ref().ok_or_else(|| {
            llm_core::EngineError::inference("none", "No active inference engine")
        })?;

        let engine = self
            .engines
            .get(engine_name)
            .ok_or_else(|| llm_core::EngineError::inference(engine_name, "Engine not found"))?;

        let engine = engine.read().await;
        engine.infer_stream(prompt, options, on_chunk).await
    }

    /// Shut down all engines.
    pub async fn shutdown_all(&self) -> Result<()> {
        for (name, engine) in &self.engines {
            info!("Shutting down engine: {}", name);
            let mut engine = engine.write().await;
            engine.shutdown().await?;
        }
        Ok(())
    }
}

impl Default for InferencePipeline {
    fn default() -> Self {
        Self::new()
    }
}
