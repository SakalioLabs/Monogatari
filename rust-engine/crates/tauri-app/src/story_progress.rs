//! Persistent, deterministic progress produced by applied story event actions.

use std::collections::{BTreeSet, HashSet};

use llm_core::normalize_script_state_key;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::story_events::{StoryEventAction, StoryEventDefinition};

pub const STORY_PROGRESS_SCHEMA_V1: &str = "monogatari-story-progress/v1";
pub const STORY_EVENT_APPLICATION_SCHEMA_V1: &str = "monogatari-story-event-application/v1";

const MAX_APPLIED_EVENT_RECORDS: usize = 4_096;
const MAX_UNLOCKED_CONTENT_IDS: usize = 16_384;
const MAX_EVENT_APPLICATION_COUNT: u32 = 1_000_000;

fn default_story_progress_schema() -> String {
    STORY_PROGRESS_SCHEMA_V1.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppliedStoryEvent {
    pub event_id: String,
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule_fingerprint: Option<String>,
    pub application_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryEventActionResult {
    pub action: StoryEventAction,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryEventApplication {
    pub schema: String,
    pub event_id: String,
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_id: Option<String>,
    pub applied: bool,
    pub application_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule_fingerprint: Option<String>,
    pub catalog_fingerprint: String,
    pub progress_fingerprint: String,
    pub action_results: Vec<StoryEventActionResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryProgressState {
    #[serde(default = "default_story_progress_schema")]
    pub schema: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog_fingerprint: Option<String>,
    #[serde(default)]
    pub applied_events: Vec<AppliedStoryEvent>,
    #[serde(default)]
    pub unlocked_scene_ids: BTreeSet<String>,
    #[serde(default)]
    pub unlocked_dialogue_ids: BTreeSet<String>,
    #[serde(default)]
    pub unlocked_ending_ids: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryProgressSnapshot {
    pub schema: String,
    pub catalog_fingerprint: Option<String>,
    pub progress_fingerprint: String,
    pub applied_event_count: usize,
    pub total_application_count: u64,
    pub applied_events: Vec<AppliedStoryEvent>,
    pub unlocked_scene_ids: Vec<String>,
    pub unlocked_dialogue_ids: Vec<String>,
    pub unlocked_ending_ids: Vec<String>,
}

impl Default for StoryProgressState {
    fn default() -> Self {
        Self {
            schema: STORY_PROGRESS_SCHEMA_V1.to_string(),
            catalog_fingerprint: None,
            applied_events: Vec::new(),
            unlocked_scene_ids: BTreeSet::new(),
            unlocked_dialogue_ids: BTreeSet::new(),
            unlocked_ending_ids: BTreeSet::new(),
        }
    }
}

impl StoryProgressState {
    pub fn validate_and_normalize(mut self) -> Result<Self, String> {
        if self.schema != STORY_PROGRESS_SCHEMA_V1 {
            return Err(format!(
                "Unsupported story progress schema `{}`; expected `{STORY_PROGRESS_SCHEMA_V1}`.",
                self.schema
            ));
        }
        if self.applied_events.len() > MAX_APPLIED_EVENT_RECORDS {
            return Err(format!(
                "Story progress contains more than {MAX_APPLIED_EVENT_RECORDS} applied event records."
            ));
        }
        for (label, count) in [
            ("scene", self.unlocked_scene_ids.len()),
            ("dialogue", self.unlocked_dialogue_ids.len()),
            ("ending", self.unlocked_ending_ids.len()),
        ] {
            if count > MAX_UNLOCKED_CONTENT_IDS {
                return Err(format!(
                    "Story progress contains more than {MAX_UNLOCKED_CONTENT_IDS} unlocked {label} ids."
                ));
            }
        }
        if let Some(fingerprint) = self.catalog_fingerprint.as_deref() {
            validate_fingerprint(fingerprint, "story progress catalog fingerprint")?;
        }

        let mut seen = HashSet::new();
        for applied in &mut self.applied_events {
            applied.event_id = normalize_progress_id(&applied.event_id, "applied event id")?;
            applied.event_type = normalize_progress_id(&applied.event_type, "applied event type")?;
            if let Some(character_id) = applied.character_id.as_mut() {
                *character_id = normalize_progress_id(character_id, "applied event character id")?;
            }
            if let Some(fingerprint) = applied.rule_fingerprint.as_deref() {
                validate_fingerprint(fingerprint, "applied event rule fingerprint")?;
            }
            if applied.application_count == 0
                || applied.application_count > MAX_EVENT_APPLICATION_COUNT
            {
                return Err(format!(
                    "Story event `{}` has an invalid application count {}.",
                    applied.event_id, applied.application_count
                ));
            }
            let key = (applied.event_id.clone(), applied.character_id.clone());
            if !seen.insert(key) {
                return Err(format!(
                    "Story progress repeats event scope `{}` / `{}`.",
                    applied.event_id,
                    applied.character_id.as_deref().unwrap_or("global")
                ));
            }
        }
        self.applied_events.sort_by(|left, right| {
            left.event_id
                .cmp(&right.event_id)
                .then_with(|| left.character_id.cmp(&right.character_id))
        });
        self.unlocked_scene_ids = normalize_unlock_set(self.unlocked_scene_ids, "scene")?;
        self.unlocked_dialogue_ids = normalize_unlock_set(self.unlocked_dialogue_ids, "dialogue")?;
        self.unlocked_ending_ids = normalize_unlock_set(self.unlocked_ending_ids, "ending")?;
        Ok(self)
    }

    pub fn has_applied(&self, event_id: &str, character_id: Option<&str>) -> bool {
        self.applied_events.iter().any(|applied| {
            applied.event_id == event_id && applied.character_id.as_deref() == character_id
        })
    }

    pub fn apply_event(
        &mut self,
        definition: &StoryEventDefinition,
        character_id: Option<&str>,
        catalog_fingerprint: &str,
    ) -> Result<StoryEventApplication, String> {
        validate_fingerprint(catalog_fingerprint, "story event catalog fingerprint")?;
        let character_id = character_id
            .map(|value| normalize_progress_id(value, "story event character id"))
            .transpose()?;
        if !definition.rule.character_ids.is_empty() {
            let Some(character_id) = character_id.as_deref() else {
                return Err(format!(
                    "Story event `{}` requires a character context.",
                    definition.event_id
                ));
            };
            if !definition.applies_to_character(character_id) {
                return Err(format!(
                    "Story event `{}` is not available for character `{character_id}`.",
                    definition.event_id
                ));
            }
        }

        let existing_index = self.applied_events.iter().position(|applied| {
            applied.event_id == definition.event_id
                && applied.character_id.as_deref() == character_id.as_deref()
        });
        let previous_count = existing_index
            .map(|index| self.applied_events[index].application_count)
            .unwrap_or(0);
        if previous_count > 0 && !definition.rule.repeatable {
            return Ok(self.application_report(
                definition,
                character_id,
                false,
                previous_count,
                catalog_fingerprint,
                Vec::new(),
            ));
        }
        let application_count = previous_count
            .checked_add(1)
            .filter(|count| *count <= MAX_EVENT_APPLICATION_COUNT)
            .ok_or_else(|| {
                format!(
                    "Story event `{}` exceeds the application count limit.",
                    definition.event_id
                )
            })?;
        if existing_index.is_none() && self.applied_events.len() >= MAX_APPLIED_EVENT_RECORDS {
            return Err(format!(
                "Story progress cannot contain more than {MAX_APPLIED_EVENT_RECORDS} applied event records."
            ));
        }

        let mut action_results = Vec::with_capacity(definition.actions.len());
        for action in &definition.actions {
            let changed = match action {
                StoryEventAction::UnlockScene { scene_id } => {
                    self.unlocked_scene_ids.insert(scene_id.clone())
                }
                StoryEventAction::UnlockDialogue { dialogue_id } => {
                    self.unlocked_dialogue_ids.insert(dialogue_id.clone())
                }
                StoryEventAction::UnlockEnding { ending_id } => {
                    self.unlocked_ending_ids.insert(ending_id.clone())
                }
                StoryEventAction::SetFlag { .. } => true,
            };
            action_results.push(StoryEventActionResult {
                action: action.clone(),
                changed,
            });
        }

        if let Some(index) = existing_index {
            self.applied_events[index].application_count = application_count;
            self.applied_events[index].event_type = definition.event_type.clone();
            self.applied_events[index].rule_fingerprint = definition.rule.rule_fingerprint.clone();
        } else {
            self.applied_events.push(AppliedStoryEvent {
                event_id: definition.event_id.clone(),
                event_type: definition.event_type.clone(),
                character_id: character_id.clone(),
                rule_fingerprint: definition.rule.rule_fingerprint.clone(),
                application_count,
            });
            self.applied_events.sort_by(|left, right| {
                left.event_id
                    .cmp(&right.event_id)
                    .then_with(|| left.character_id.cmp(&right.character_id))
            });
        }
        self.catalog_fingerprint = Some(catalog_fingerprint.to_string());

        Ok(self.application_report(
            definition,
            character_id,
            true,
            application_count,
            catalog_fingerprint,
            action_results,
        ))
    }

    pub fn snapshot(&self) -> StoryProgressSnapshot {
        StoryProgressSnapshot {
            schema: self.schema.clone(),
            catalog_fingerprint: self.catalog_fingerprint.clone(),
            progress_fingerprint: self.progress_fingerprint(),
            applied_event_count: self.applied_events.len(),
            total_application_count: self
                .applied_events
                .iter()
                .map(|applied| u64::from(applied.application_count))
                .sum(),
            applied_events: self.applied_events.clone(),
            unlocked_scene_ids: self.unlocked_scene_ids.iter().cloned().collect(),
            unlocked_dialogue_ids: self.unlocked_dialogue_ids.iter().cloned().collect(),
            unlocked_ending_ids: self.unlocked_ending_ids.iter().cloned().collect(),
        }
    }

    pub fn progress_fingerprint(&self) -> String {
        let encoded = serde_json::to_vec(&json!({
            "schema": STORY_PROGRESS_SCHEMA_V1,
            "catalog_fingerprint": self.catalog_fingerprint,
            "applied_events": self.applied_events,
            "unlocked_scene_ids": self.unlocked_scene_ids,
            "unlocked_dialogue_ids": self.unlocked_dialogue_ids,
            "unlocked_ending_ids": self.unlocked_ending_ids,
        }))
        .expect("story progress fingerprint payload should serialize");
        let mut hasher = Sha256::new();
        hasher.update(encoded);
        format!("{:x}", hasher.finalize())
    }

    fn application_report(
        &self,
        definition: &StoryEventDefinition,
        character_id: Option<String>,
        applied: bool,
        application_count: u32,
        catalog_fingerprint: &str,
        action_results: Vec<StoryEventActionResult>,
    ) -> StoryEventApplication {
        StoryEventApplication {
            schema: STORY_EVENT_APPLICATION_SCHEMA_V1.to_string(),
            event_id: definition.event_id.clone(),
            event_type: definition.event_type.clone(),
            character_id,
            applied,
            application_count,
            rule_fingerprint: definition.rule.rule_fingerprint.clone(),
            catalog_fingerprint: catalog_fingerprint.to_string(),
            progress_fingerprint: self.progress_fingerprint(),
            action_results,
        }
    }
}

fn normalize_progress_id(value: &str, label: &str) -> Result<String, String> {
    let normalized = normalize_script_state_key(value)
        .map_err(|error| format!("Invalid {label} `{value}`: {error}"))?;
    if normalized != value {
        return Err(format!(
            "Invalid {label} `{value}`: surrounding whitespace is not allowed."
        ));
    }
    Ok(normalized)
}

fn normalize_unlock_set(values: BTreeSet<String>, label: &str) -> Result<BTreeSet<String>, String> {
    values
        .into_iter()
        .map(|value| normalize_progress_id(&value, &format!("unlocked {label} id")))
        .collect()
}

fn validate_fingerprint(value: &str, label: &str) -> Result<(), String> {
    if value.len() != 64
        || !value
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
    {
        return Err(format!("Invalid {label}."));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story_events::{EventTriggerRule, StoryEventDefinition};
    use serde_json::json;

    fn definition(repeatable: bool) -> StoryEventDefinition {
        StoryEventDefinition {
            event_id: "luna_secret".to_string(),
            event_type: "unlock".to_string(),
            description: "Unlock Luna's secret".to_string(),
            data: json!({}),
            actions: vec![
                StoryEventAction::UnlockScene {
                    scene_id: "moon_garden".to_string(),
                },
                StoryEventAction::SetFlag {
                    flag: "luna.secret".to_string(),
                    value: true,
                },
            ],
            rule: EventTriggerRule {
                event_id: "luna_secret".to_string(),
                event_type: "unlock".to_string(),
                rule_fingerprint: Some("a".repeat(64)),
                min_relationship: None,
                score_metric: None,
                min_score: None,
                min_evaluation_count: None,
                character_ids: vec!["luna".to_string()],
                repeatable,
            },
            source_path: "events/luna.json".to_string(),
        }
    }

    #[test]
    fn nonrepeatable_event_applies_once_per_character_scope() {
        let mut progress = StoryProgressState::default();
        let fingerprint = "b".repeat(64);

        let first = progress
            .apply_event(&definition(false), Some("luna"), &fingerprint)
            .unwrap();
        let second = progress
            .apply_event(&definition(false), Some("luna"), &fingerprint)
            .unwrap();

        assert!(first.applied);
        assert!(!second.applied);
        assert_eq!(second.application_count, 1);
        assert!(progress.unlocked_scene_ids.contains("moon_garden"));
        assert_eq!(progress.applied_events.len(), 1);
    }

    #[test]
    fn repeatable_event_increments_count_but_unlocks_idempotently() {
        let mut progress = StoryProgressState::default();
        let fingerprint = "c".repeat(64);

        let first = progress
            .apply_event(&definition(true), Some("luna"), &fingerprint)
            .unwrap();
        let second = progress
            .apply_event(&definition(true), Some("luna"), &fingerprint)
            .unwrap();

        assert!(first.action_results[0].changed);
        assert!(!second.action_results[0].changed);
        assert_eq!(second.application_count, 2);
    }

    #[test]
    fn scoped_event_rejects_missing_or_wrong_character() {
        let mut progress = StoryProgressState::default();
        let fingerprint = "d".repeat(64);

        assert!(progress
            .apply_event(&definition(false), None, &fingerprint)
            .is_err());
        assert!(progress
            .apply_event(&definition(false), Some("sakura"), &fingerprint)
            .is_err());
        assert!(progress.applied_events.is_empty());
    }

    #[test]
    fn persisted_progress_rejects_duplicate_scopes_and_invalid_counts() {
        let applied = AppliedStoryEvent {
            event_id: "same".to_string(),
            event_type: "unlock".to_string(),
            character_id: None,
            rule_fingerprint: None,
            application_count: 1,
        };
        let mut progress = StoryProgressState {
            applied_events: vec![applied.clone(), applied],
            ..StoryProgressState::default()
        };
        assert!(progress.clone().validate_and_normalize().is_err());

        progress.applied_events.truncate(1);
        progress.applied_events[0].application_count = 0;
        assert!(progress.validate_and_normalize().is_err());
    }
}
