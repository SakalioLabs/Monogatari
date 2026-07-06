//! Character personality system based on Big Five traits.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Big Five personality traits, each on a 0.0 to 1.0 scale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    /// Openness to experience (curious vs. cautious).
    pub openness: f32,
    /// Conscientiousness (organized vs. careless).
    pub conscientiousness: f32,
    /// Extraversion (outgoing vs. reserved).
    pub extraversion: f32,
    /// Agreeableness (friendly vs. challenging).
    pub agreeableness: f32,
    /// Neuroticism (sensitive vs. confident).
    pub neuroticism: f32,
    /// Things this character likes.
    pub likes: Vec<String>,
    /// Things this character dislikes.
    pub dislikes: Vec<String>,
    /// How this character speaks (e.g., "formal", "casual", "shy").
    pub speech_style: String,
    /// Current emotional state.
    pub current_emotion: String,
    /// Intensity of current emotion (0.0 to 1.0).
    pub emotion_intensity: f32,
    /// Map of emotion names to available sprite/model states.
    pub emotion_states: HashMap<String, Vec<String>>,
}

impl Default for Personality {
    fn default() -> Self {
        let mut emotion_states = HashMap::new();
        emotion_states.insert(
            "neutral".to_string(),
            vec!["default".to_string()],
        );

        Self {
            openness: 0.5,
            conscientiousness: 0.5,
            extraversion: 0.5,
            agreeableness: 0.5,
            neuroticism: 0.5,
            likes: Vec::new(),
            dislikes: Vec::new(),
            speech_style: "casual".to_string(),
            current_emotion: "neutral".to_string(),
            emotion_intensity: 0.5,
            emotion_states,
        }
    }
}

impl Personality {
    /// Build a text description of this personality for LLM context.
    pub fn to_prompt_description(&self) -> String {
        let mut parts = vec![
            format!("Speech style: {}", self.speech_style),
            format!("Current emotion: {} (intensity: {:.1})", self.current_emotion, self.emotion_intensity),
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
