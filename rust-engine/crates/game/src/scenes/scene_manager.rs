//! Stack-based scene manager.

use tracing::{debug, info};

use llm_core::Result;

use super::scene::{InputEvent, Scene, SceneRenderState};

/// Stack-based scene manager.
///
/// Scenes are stored in a stack. The top scene receives input and updates.
/// Drawing goes back-to-front, stopping at the first opaque scene.
pub struct SceneManager {
    scenes: Vec<Box<dyn Scene>>,
}

impl SceneManager {
    /// Create a new empty scene manager.
    pub fn new() -> Self {
        Self { scenes: Vec::new() }
    }

    /// Push a new scene onto the stack.
    pub async fn push_scene(&mut self, mut scene: Box<dyn Scene>) -> Result<()> {
        // Pause the current top scene
        if let Some(top) = self.scenes.last_mut() {
            debug!("Pausing scene: {}", top.name());
            top.on_pause().await?;
        }

        let name = scene.name().to_string();
        info!("Pushing scene: {}", name);
        scene.on_enter().await?;
        self.scenes.push(scene);
        Ok(())
    }

    /// Pop the top scene off the stack.
    pub async fn pop_scene(&mut self) -> Result<()> {
        if let Some(mut scene) = self.scenes.pop() {
            info!("Popping scene: {}", scene.name());
            scene.on_exit().await?;
        }

        // Resume the new top scene
        if let Some(top) = self.scenes.last_mut() {
            debug!("Resuming scene: {}", top.name());
            top.on_resume().await?;
        }

        Ok(())
    }

    /// Replace the top scene with a new one.
    pub async fn replace_scene(&mut self, scene: Box<dyn Scene>) -> Result<()> {
        self.pop_scene().await?;
        self.push_scene(scene).await
    }

    /// Update the top scene.
    pub async fn update(&mut self, delta_time: f32) -> Result<()> {
        if let Some(top) = self.scenes.last_mut() {
            top.update(delta_time).await?;
        }
        Ok(())
    }

    /// Draw scenes back-to-front, stopping at the first opaque scene.
    pub async fn draw(&self) -> Result<Vec<SceneRenderState>> {
        let mut states = Vec::new();

        for scene in self.scenes.iter().rev() {
            states.push(scene.draw().await?);
            if scene.is_opaque() {
                break;
            }
        }

        states.reverse();
        Ok(states)
    }

    /// Send an input event to the top scene.
    pub async fn handle_input(&mut self, event: InputEvent) -> Result<bool> {
        if let Some(top) = self.scenes.last_mut() {
            top.handle_input(event).await
        } else {
            Ok(false)
        }
    }

    /// Get the name of the current top scene.
    pub fn current_scene_name(&self) -> Option<&str> {
        self.scenes.last().map(|s| s.name())
    }

    /// Get the number of scenes on the stack.
    pub fn scene_count(&self) -> usize {
        self.scenes.len()
    }

    /// Check if the scene stack is empty.
    pub fn is_empty(&self) -> bool {
        self.scenes.is_empty()
    }
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new()
    }
}
