//! Character personality system based on Big Five traits.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Big Five personality traits, each on a 0.0 to 1.0 scale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    /// Openness to experience (curious vs. cautious).
    #[serde(default = "default_trait_score")]
    pub openness: f32,
    /// Conscientiousness (organized vs. careless).
    #[serde(default = "default_trait_score")]
    pub conscientiousness: f32,
    /// Extraversion (outgoing vs. reserved).
    #[serde(default = "default_trait_score")]
    pub extraversion: f32,
    /// Agreeableness (friendly vs. challenging).
    #[serde(default = "default_trait_score")]
    pub agreeableness: f32,
    /// Neuroticism (sensitive vs. confident).
    #[serde(default = "default_trait_score")]
    pub neuroticism: f32,
    /// Things this character likes.
    #[serde(default)]
    pub likes: Vec<String>,
    /// Things this character dislikes.
    #[serde(default)]
    pub dislikes: Vec<String>,
    /// How this character speaks (e.g., "formal", "casual", "shy").
    #[serde(default = "default_speech_style")]
    pub speech_style: String,
    /// Current emotional state.
    #[serde(default = "default_current_emotion")]
    pub current_emotion: String,
    /// Intensity of current emotion (0.0 to 1.0).
    #[serde(default = "default_emotion_intensity")]
    pub emotion_intensity: f32,
    /// Map of emotion names to available sprite/model states.
    #[serde(default = "default_emotion_states")]
    pub emotion_states: HashMap<String, Vec<String>>,
}

fn default_trait_score() -> f32 {
    0.5
}

fn default_speech_style() -> String {
    "casual".to_string()
}

fn default_current_emotion() -> String {
    "neutral".to_string()
}

fn default_emotion_intensity() -> f32 {
    0.5
}

fn default_emotion_states() -> HashMap<String, Vec<String>> {
    let mut emotion_states = HashMap::new();
    emotion_states.insert("neutral".to_string(), vec!["default".to_string()]);
    emotion_states
}

impl Default for Personality {
    fn default() -> Self {
        Self {
            openness: default_trait_score(),
            conscientiousness: default_trait_score(),
            extraversion: default_trait_score(),
            agreeableness: default_trait_score(),
            neuroticism: default_trait_score(),
            likes: Vec::new(),
            dislikes: Vec::new(),
            speech_style: default_speech_style(),
            current_emotion: default_current_emotion(),
            emotion_intensity: default_emotion_intensity(),
            emotion_states: default_emotion_states(),
        }
    }
}

impl Personality {
    /// Build a text description of this personality for LLM context.
    pub fn to_prompt_description(&self) -> String {
        let mut parts = vec![
            format!("Speech style: {}", self.speech_style),
            format!(
                "Current emotion: {} (intensity: {:.1})",
                self.current_emotion, self.emotion_intensity
            ),
        ];

        if !self.likes.is_empty() {
            parts.push(format!("Likes: {}", self.likes.join(", ")));
        }
        if !self.dislikes.is_empty() {
            parts.push(format!("Dislikes: {}", self.dislikes.join(", ")));
        }

        parts.push(format!(
            "Personality traits: Openness={:.1}, Conscientiousness={:.1}, Extraversion={:.1}, Agreeableness={:.1}, Neuroticism={:.1}",
            self.openness, self.conscientiousness, self.extraversion, self.agreeableness, self.neuroticism
        ));

        parts.join("\n")
    }

    /// Get available sprite/model states for the current emotion.
    pub fn current_emotion_states(&self) -> &[String] {
        self.emotion_states
            .get(&self.current_emotion)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}
