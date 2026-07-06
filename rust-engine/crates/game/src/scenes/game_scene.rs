//! Main gameplay scene.

use async_trait::async_trait;
use tracing::debug;

use llm_core::Result;

use super::scene::{InputEvent, Scene, SceneRenderState};

/// State of the dialogue display.
#[derive(Debug, Clone)]
pub enum DialogueDisplayState {
    /// Showing dialogue text.
    ShowingText {
        speaker: Option<String>,
        text: String,
        emotion: Option<String>,
    },
    /// Showing choices to the player.
    ShowingChoices {
        speaker: Option<String>,
        text: String,
        choices: Vec<String>,
        selected: usize,
    },
    /// No dialogue active (waiting).
    Idle,
}

/// Main gameplay scene for dialogue and character interaction.
pub struct GameScene {
    display_state: DialogueDisplayState,
    /// ID of the dialogue to start when entering the scene.
    start_dialogue_id: Option<String>,
    /// Current character speaker name.
    current_speaker: Option<String>,
    /// Current character emotion.
    current_emotion: Option<String>,
    /// Live2D model expression/motion to play.
    live2d_expression: Option<String>,
    live2d_motion: Option<String>,
}

impl GameScene {
    /// Create a new game scene.
    pub fn new() -> Self {
        Self {
            display_state: DialogueDisplayState::Idle,
            start_dialogue_id: None,
            current_speaker: None,
            current_emotion: None,
            live2d_expression: None,
            live2d_motion: None,
        }
    }

    /// Create a game scene that starts a specific dialogue.
    pub fn with_dialogue(dialogue_id: impl Into<String>) -> Self {
        Self {
            start_dialogue_id: Some(dialogue_id.into()),
            ..Self::new()
        }
    }

    /// Update the displayed dialogue text.
    pub fn set_dialogue_text(
        &mut self,
        speaker: Option<String>,
        text: String,
        emotion: Option<String>,
    ) {
        self.current_speaker = speaker.clone();
        self.current_emotion = emotion.clone();
        self.display_state = DialogueDisplayState::ShowingText {
            speaker,
            text,
            emotion,
        };
    }

    /// Show choices to the player.
    pub fn show_choices(&mut self, choices: Vec<String>) {
        if let DialogueDisplayState::ShowingText { speaker, text, .. } = &self.display_state {
            self.display_state = DialogueDisplayState::ShowingChoices {
                speaker: speaker.clone(),
                text: text.clone(),
                choices,
                selected: 0,
            };
        }
    }

    /// Set the Live2D model expression.
    pub fn set_live2d_expression(&mut self, expression: Option<String>) {
        self.live2d_expression = expression;
    }

    /// Set the Live2D model motion.
    pub fn set_live2d_motion(&mut self, motion: Option<String>) {
        self.live2d_motion = motion;
    }
}

#[async_trait]
impl Scene for GameScene {
    fn name(&self) -> &str {
        "GameScene"
    }

    fn is_opaque(&self) -> bool {
        true
    }

    async fn on_enter(&mut self) -> Result<()> {
        debug!("GameScene entered");
        if let Some(dialogue_id) = &self.start_dialogue_id {
            debug!("Starting dialogue: {}", dialogue_id);
            // In full implementation, this would trigger DialogueManager
        }
        Ok(())
    }

    async fn update(&mut self, _delta_time: f32) -> Result<()> {
        Ok(())
    }

    async fn draw(&self) -> Result<SceneRenderState> {
        let data = match &self.display_state {
            DialogueDisplayState::ShowingText {
                speaker,
                text,
                emotion,
            } => {
                serde_json::json!({
                    "state": "text",
                    "speaker": speaker,
                    "text": text,
                    "emotion": emotion,
                    "live2d_expression": self.live2d_expression,
                    "live2d_motion": self.live2d_motion,
                })
            }
            DialogueDisplayState::ShowingChoices {
                speaker,
                text,
                choices,
                selected,
            } => {
                serde_json::json!({
                    "state": "choices",
                    "speaker": speaker,
                    "text": text,
                    "choices": choices,
                    "selected": selected,
                })
            }
            DialogueDisplayState::Idle => {
                serde_json::json!({
                    "state": "idle",
                })
            }
        };

        Ok(SceneRenderState {
            scene_type: "game".to_string(),
            data,
        })
    }

    async fn handle_input(&mut self, event: InputEvent) -> Result<bool> {
        match event {
            InputEvent::KeyPress { key } => match key.as_str() {
                " " | "Enter" => {
                    // Advance dialogue
                    debug!("Advance dialogue");
                    Ok(true)
                }
                "Escape" => {
                    // Go back / open menu
                    debug!("Escape pressed");
                    Ok(true)
                }
                _ => Ok(false),
            },
            InputEvent::ChoiceSelect { index } => {
                if let DialogueDisplayState::ShowingChoices { choices, selected, .. } =
                    &mut self.display_state
                {
                    if index < choices.len() {
                        *selected = index;
                        debug!("Selected choice: {}", index);
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }
}

impl Default for GameScene {
    fn default() -> Self {
        Self::new()
    }
}
