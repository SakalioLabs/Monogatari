//! Core traits for the engine.

use async_trait::async_trait;

use crate::Result;

/// Trait for game services that can be registered with the engine.
///
/// Services are initialized once, updated each frame, and shut down on exit.
#[async_trait]
pub trait GameService: Send + Sync {
    /// Name of this service for logging/debugging.
    fn name(&self) -> &str;

    /// Called once when the service is first registered.
    async fn initialize(&mut self) -> Result<()>;

    /// Called every frame with the delta time in seconds.
    async fn update(&mut self, delta_time: f32) -> Result<()>;

    /// Called when the engine is shutting down.
    async fn shutdown(&mut self) -> Result<()>;
}
