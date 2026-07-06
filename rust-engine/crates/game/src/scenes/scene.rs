//! Scene trait definition.

use async_trait::async_trait;

use llm_core::Result;

/// Trait for game scenes.
///
/// Scenes are managed in a stack. The top scene receives input and is drawn.
/// Opaque scenes prevent scenes below from being drawn.
#[async_trait]
pub trait Scene: Send + Sync {
    /// Name of this scene for debugging.
    fn name(&self) -> &str;

    /// Whether this scene is opaque (hides scenes below).
    fn is_opaque(&self) -> bool {
        true
    }

    /// Called when the scene is pushed onto the stack.
    async fn on_enter(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when the scene is popped from the stack.
    async fn on_exit(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when another scene is pushed on top of this one.
    async fn on_pause(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when this scene becomes the top scene again.
    async fn on_resume(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called every frame with delta time in seconds.
    async fn update(&mut self, delta_time: f32) -> Result<()>;

    /// Render the scene. In Tauri, this sends state to the frontend.
    async fn draw(&self) -> Result<SceneRenderState>;

    /// Handle an input event. Returns true if the event was consumed.
    async fn handle_input(&mut self, event: InputEvent) -> Result<bool>;
}

/// State sent to the frontend for rendering.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SceneRenderState {
    /// Name of the scene type.
    pub scene_type: String,
    /// Custom data for the frontend to interpret.
    pub data: serde_json::Value,
}

/// Input events from the frontend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InputEvent {
    /// Mouse click at position.
    Click { x: f32, y: f32 },
    /// Keyboard key pressed.
    KeyPress { key: String },
    /// Keyboard key released.
    KeyRelease { key: String },
    /// Mouse moved.
    MouseMove { x: f32, y: f32 },
    /// Choice selected (by index).
    ChoiceSelect { index: usize },
    /// Custom action from frontend.
    Action { name: String, data: serde_json::Value },
}
