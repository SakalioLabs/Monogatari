//! Title screen scene.

use async_trait::async_trait;

use llm_core::Result;

use super::scene::{InputEvent, Scene, SceneRenderState};

/// Title screen with Start Game, Settings, and Exit options.
pub struct TitleScene {
    selected_option: usize,
    options: Vec<String>,
}

impl TitleScene {
    /// Create a new title scene.
    pub fn new() -> Self {
        Self {
            selected_option: 0,
            options: vec![
                "Start Game".to_string(),
                "Settings".to_string(),
                "Exit".to_string(),
            ],
        }
    }
}

#[async_trait]
impl Scene for TitleScene {
    fn name(&self) -> &str {
        "TitleScene"
    }

    fn is_opaque(&self) -> bool {
        true
    }

    async fn update(&mut self, _delta_time: f32) -> Result<()> {
        Ok(())
    }

    async fn draw(&self) -> Result<SceneRenderState> {
        Ok(SceneRenderState {
            scene_type: "title".to_string(),
            data: serde_json::json!({
                "title": "LLM Galgame Engine",
                "options": self.options,
                "selected": self.selected_option,
            }),
        })
    }

    async fn handle_input(&mut self, event: InputEvent) -> Result<bool> {
        match event {
            InputEvent::KeyPress { key } => match key.as_str() {
                "ArrowUp" | "w" => {
                    self.selected_option = if self.selected_option == 0 {
                        self.options.len() - 1
                    } else {
                        self.selected_option - 1
                    };
                    Ok(true)
                }
                "ArrowDown" | "s" => {
                    self.selected_option = (self.selected_option + 1) % self.options.len();
                    Ok(true)
                }
                "Enter" | " " => {
                    // Return the selected option via an Action event
                    Ok(true)
                }
                _ => Ok(false),
            },
            InputEvent::ChoiceSelect { index } => {
                if index < self.options.len() {
                    self.selected_option = index;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }
}

impl Default for TitleScene {
    fn default() -> Self {
        Self::new()
    }
}
