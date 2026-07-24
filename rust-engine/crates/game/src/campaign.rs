//! Deterministic sequencing for multi-roleplay AI visual-novel campaigns.

use std::collections::{BTreeMap, HashMap, HashSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::scene_roleplay::{SceneRoleplayDefinition, SceneRoleplaySession, SceneRoleplayStatus};

pub const ROLEPLAY_CAMPAIGN_SCHEMA_V1: &str = "monogatari-roleplay-campaign/v1";
pub const ROLEPLAY_CAMPAIGN_SESSION_SCHEMA_V1: &str = "monogatari-roleplay-campaign-session/v1";

const MAX_CAMPAIGN_ENTRIES: usize = 256;
const MAX_ENTRY_ROUTES: usize = 64;
const MAX_COMPLETED_ENTRIES: usize = 256;
const MAX_SUMMARY_VALUES: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RoleplayCampaignDefinition {
    pub schema: String,
    pub id: String,
    pub title: String,
    pub start_entry_id: String,
    pub entries: Vec<RoleplayCampaignEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RoleplayCampaignEntry {
    pub id: String,
    pub roleplay_id: String,
    pub routes: Vec<RoleplayCampaignRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoleplayCampaignRoute {
    pub ending_id: String,
    pub target: RoleplayCampaignTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RoleplayCampaignTarget {
    Entry { entry_id: String },
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoleplayCampaignStatus {
    Active,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RoleplayCampaignSession {
    pub schema: String,
    pub campaign_id: String,
    pub current_entry_id: Option<String>,
    pub status: RoleplayCampaignStatus,
    #[serde(default)]
    pub relationships: BTreeMap<String, f32>,
    #[serde(default)]
    pub completed_entries: Vec<RoleplayCampaignCompletion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RoleplayCampaignCompletion {
    pub entry_id: String,
    pub roleplay_id: String,
    pub ending_id: String,
    pub total_turns: u32,
    pub scores: BTreeMap<String, f32>,
    pub observed_evidence: Vec<String>,
    pub relationships: BTreeMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RoleplayCampaignAdvance {
    pub completed: RoleplayCampaignCompletion,
    pub target: RoleplayCampaignTarget,
    pub status: RoleplayCampaignStatus,
    pub current_entry_id: Option<String>,
    pub relationships: BTreeMap<String, f32>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RoleplayCampaignError {
    #[error("invalid roleplay campaign definition: {0}")]
    InvalidDefinition(String),
    #[error("invalid roleplay campaign session: {0}")]
    InvalidSession(String),
    #[error("roleplay campaign session is already completed")]
    SessionCompleted,
    #[error("roleplay campaign entry `{0}` is not active")]
    EntryMismatch(String),
    #[error("completed roleplay does not match campaign entry: {0}")]
    RoleplayMismatch(String),
    #[error("roleplay ending `{0}` has no campaign route")]
    EndingNotRouted(String),
}

impl RoleplayCampaignDefinition {
    pub fn validate(&self) -> Result<(), RoleplayCampaignError> {
        if self.schema != ROLEPLAY_CAMPAIGN_SCHEMA_V1 {
            return invalid_definition(format!("schema must be `{ROLEPLAY_CAMPAIGN_SCHEMA_V1}`"));
        }
        bounded_id(&self.id, "campaign id")?;
        bounded_text(&self.title, "campaign title", 256)?;
        bounded_id(&self.start_entry_id, "campaign start entry id")?;
        if self.entries.is_empty() || self.entries.len() > MAX_CAMPAIGN_ENTRIES {
            return invalid_definition(format!(
                "entries must contain between 1 and {MAX_CAMPAIGN_ENTRIES} items"
            ));
        }

        let mut entry_ids = HashSet::new();
        let mut roleplay_ids = HashSet::new();
        for entry in &self.entries {
            bounded_id(&entry.id, "campaign entry id")?;
            bounded_id(&entry.roleplay_id, "campaign roleplay id")?;
            if !entry_ids.insert(entry.id.as_str()) {
                return invalid_definition(format!("duplicate entry id `{}`", entry.id));
            }
            if !roleplay_ids.insert(entry.roleplay_id.as_str()) {
                return invalid_definition(format!(
                    "roleplay `{}` is assigned to more than one campaign entry",
                    entry.roleplay_id
                ));
            }
            if entry.routes.is_empty() || entry.routes.len() > MAX_ENTRY_ROUTES {
                return invalid_definition(format!(
                    "entry `{}` routes must contain between 1 and {MAX_ENTRY_ROUTES} items",
                    entry.id
                ));
            }
            let mut ending_ids = HashSet::new();
            for route in &entry.routes {
                bounded_id(&route.ending_id, "campaign route ending id")?;
                if !ending_ids.insert(route.ending_id.as_str()) {
                    return invalid_definition(format!(
                        "entry `{}` repeats ending route `{}`",
                        entry.id, route.ending_id
                    ));
                }
                if let RoleplayCampaignTarget::Entry { entry_id } = &route.target {
                    bounded_id(entry_id, "campaign route target entry id")?;
                }
            }
        }
        if !entry_ids.contains(self.start_entry_id.as_str()) {
            return invalid_definition(format!(
                "start entry `{}` does not exist",
                self.start_entry_id
            ));
        }
        for entry in &self.entries {
            for route in &entry.routes {
                if let RoleplayCampaignTarget::Entry { entry_id } = &route.target {
                    if !entry_ids.contains(entry_id.as_str()) {
                        return invalid_definition(format!(
                            "entry `{}` targets missing entry `{entry_id}`",
                            entry.id
                        ));
                    }
                }
            }
        }

        let entries = self
            .entries
            .iter()
            .map(|entry| (entry.id.as_str(), entry))
            .collect::<HashMap<_, _>>();
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut has_completion = false;
        validate_reachable_graph(
            &self.start_entry_id,
            &entries,
            &mut visiting,
            &mut visited,
            &mut has_completion,
        )?;
        if visited.len() != self.entries.len() {
            let mut unreachable = self
                .entries
                .iter()
                .filter(|entry| !visited.contains(entry.id.as_str()))
                .map(|entry| entry.id.as_str())
                .collect::<Vec<_>>();
            unreachable.sort_unstable();
            return invalid_definition(format!("unreachable entries: {}", unreachable.join(", ")));
        }
        if !has_completion {
            return invalid_definition("campaign has no completion route");
        }
        Ok(())
    }

    pub fn entry(&self, entry_id: &str) -> Option<&RoleplayCampaignEntry> {
        self.entries.iter().find(|entry| entry.id == entry_id)
    }
}

impl RoleplayCampaignSession {
    pub fn start(definition: &RoleplayCampaignDefinition) -> Result<Self, RoleplayCampaignError> {
        Self::start_with_relationships(definition, BTreeMap::new())
    }

    pub fn start_with_relationships(
        definition: &RoleplayCampaignDefinition,
        relationships: BTreeMap<String, f32>,
    ) -> Result<Self, RoleplayCampaignError> {
        definition.validate()?;
        Ok(Self {
            schema: ROLEPLAY_CAMPAIGN_SESSION_SCHEMA_V1.to_string(),
            campaign_id: definition.id.clone(),
            current_entry_id: Some(definition.start_entry_id.clone()),
            status: RoleplayCampaignStatus::Active,
            relationships: normalize_relationships(relationships)?,
            completed_entries: Vec::new(),
        })
    }

    pub fn validate(
        &self,
        definition: &RoleplayCampaignDefinition,
    ) -> Result<(), RoleplayCampaignError> {
        definition.validate()?;
        if self.schema != ROLEPLAY_CAMPAIGN_SESSION_SCHEMA_V1 {
            return invalid_session(format!(
                "schema must be `{ROLEPLAY_CAMPAIGN_SESSION_SCHEMA_V1}`"
            ));
        }
        if self.campaign_id != definition.id {
            return invalid_session(format!(
                "campaign id `{}` does not match `{}`",
                self.campaign_id, definition.id
            ));
        }
        if self.completed_entries.len() > MAX_COMPLETED_ENTRIES {
            return invalid_session(format!("completed entries exceed {MAX_COMPLETED_ENTRIES}"));
        }
        normalize_relationships(self.relationships.clone())?;

        let mut expected_entry_id = Some(definition.start_entry_id.as_str());
        let mut expected_status = RoleplayCampaignStatus::Active;
        let mut seen = HashSet::new();
        for completion in &self.completed_entries {
            validate_completion(completion)?;
            let Some(expected) = expected_entry_id else {
                return invalid_session("completion appears after campaign end");
            };
            if completion.entry_id != expected {
                return invalid_session(format!(
                    "expected completion for entry `{expected}`, found `{}`",
                    completion.entry_id
                ));
            }
            if !seen.insert(completion.entry_id.as_str()) {
                return invalid_session(format!(
                    "entry `{}` is completed more than once",
                    completion.entry_id
                ));
            }
            let entry = definition
                .entry(expected)
                .ok_or_else(|| RoleplayCampaignError::InvalidSession(expected.to_string()))?;
            if completion.roleplay_id != entry.roleplay_id {
                return invalid_session(format!(
                    "entry `{expected}` completion uses roleplay `{}`",
                    completion.roleplay_id
                ));
            }
            let route = entry
                .routes
                .iter()
                .find(|route| route.ending_id == completion.ending_id)
                .ok_or_else(|| {
                    RoleplayCampaignError::EndingNotRouted(completion.ending_id.clone())
                })?;
            match &route.target {
                RoleplayCampaignTarget::Entry { entry_id } => {
                    expected_entry_id = Some(entry_id);
                    expected_status = RoleplayCampaignStatus::Active;
                }
                RoleplayCampaignTarget::Complete => {
                    expected_entry_id = None;
                    expected_status = RoleplayCampaignStatus::Completed;
                }
            }
        }

        if self.status != expected_status {
            return invalid_session("status does not match completed route history");
        }
        if self.current_entry_id.as_deref() != expected_entry_id {
            return invalid_session("current entry does not match completed route history");
        }
        Ok(())
    }

    pub fn complete_current_entry(
        &mut self,
        definition: &RoleplayCampaignDefinition,
        roleplay_definition: &SceneRoleplayDefinition,
        roleplay_session: &SceneRoleplaySession,
    ) -> Result<RoleplayCampaignAdvance, RoleplayCampaignError> {
        self.validate(definition)?;
        if self.status == RoleplayCampaignStatus::Completed {
            return Err(RoleplayCampaignError::SessionCompleted);
        }
        let entry_id = self
            .current_entry_id
            .as_deref()
            .ok_or(RoleplayCampaignError::SessionCompleted)?;
        let entry = definition
            .entry(entry_id)
            .ok_or_else(|| RoleplayCampaignError::EntryMismatch(entry_id.to_string()))?;
        if roleplay_definition.id != entry.roleplay_id {
            return Err(RoleplayCampaignError::RoleplayMismatch(format!(
                "entry `{entry_id}` requires `{}`, received definition `{}`",
                entry.roleplay_id, roleplay_definition.id
            )));
        }
        roleplay_definition
            .validate()
            .map_err(|error| RoleplayCampaignError::RoleplayMismatch(error.to_string()))?;
        roleplay_session
            .validate_snapshot(roleplay_definition)
            .map_err(|error| RoleplayCampaignError::RoleplayMismatch(error.to_string()))?;
        if roleplay_session.roleplay_id != entry.roleplay_id {
            return Err(RoleplayCampaignError::RoleplayMismatch(format!(
                "entry `{entry_id}` requires `{}`, received session `{}`",
                entry.roleplay_id, roleplay_session.roleplay_id
            )));
        }
        if roleplay_session.status != SceneRoleplayStatus::Completed {
            return Err(RoleplayCampaignError::RoleplayMismatch(
                "roleplay session is not completed".to_string(),
            ));
        }
        let ending_id = roleplay_session.ending_id.as_deref().ok_or_else(|| {
            RoleplayCampaignError::RoleplayMismatch(
                "completed roleplay session has no ending".to_string(),
            )
        })?;
        let route = entry
            .routes
            .iter()
            .find(|route| route.ending_id == ending_id)
            .ok_or_else(|| RoleplayCampaignError::EndingNotRouted(ending_id.to_string()))?
            .clone();

        let completion = RoleplayCampaignCompletion {
            entry_id: entry.id.clone(),
            roleplay_id: entry.roleplay_id.clone(),
            ending_id: ending_id.to_string(),
            total_turns: roleplay_session.total_turns,
            scores: bounded_values(&roleplay_session.scores, "score")?,
            observed_evidence: bounded_evidence(&roleplay_session.observed_evidence)?,
            relationships: normalize_relationships(roleplay_session.relationships.clone())?,
        };
        for (character_id, value) in &completion.relationships {
            self.relationships.insert(character_id.clone(), *value);
        }
        self.completed_entries.push(completion.clone());
        match &route.target {
            RoleplayCampaignTarget::Entry { entry_id } => {
                self.current_entry_id = Some(entry_id.clone());
                self.status = RoleplayCampaignStatus::Active;
            }
            RoleplayCampaignTarget::Complete => {
                self.current_entry_id = None;
                self.status = RoleplayCampaignStatus::Completed;
            }
        }
        self.validate(definition)?;

        Ok(RoleplayCampaignAdvance {
            completed: completion,
            target: route.target,
            status: self.status.clone(),
            current_entry_id: self.current_entry_id.clone(),
            relationships: self.relationships.clone(),
        })
    }
}

fn validate_reachable_graph<'a>(
    entry_id: &'a str,
    entries: &HashMap<&'a str, &'a RoleplayCampaignEntry>,
    visiting: &mut HashSet<&'a str>,
    visited: &mut HashSet<&'a str>,
    has_completion: &mut bool,
) -> Result<(), RoleplayCampaignError> {
    if visited.contains(entry_id) {
        return Ok(());
    }
    if !visiting.insert(entry_id) {
        return invalid_definition(format!("campaign contains a cycle at `{entry_id}`"));
    }
    let entry = entries
        .get(entry_id)
        .ok_or_else(|| RoleplayCampaignError::InvalidDefinition(entry_id.to_string()))?;
    for route in &entry.routes {
        match &route.target {
            RoleplayCampaignTarget::Entry { entry_id } => {
                validate_reachable_graph(entry_id, entries, visiting, visited, has_completion)?;
            }
            RoleplayCampaignTarget::Complete => *has_completion = true,
        }
    }
    visiting.remove(entry_id);
    visited.insert(entry_id);
    Ok(())
}

fn validate_completion(
    completion: &RoleplayCampaignCompletion,
) -> Result<(), RoleplayCampaignError> {
    bounded_id(&completion.entry_id, "completed entry id")?;
    bounded_id(&completion.roleplay_id, "completed roleplay id")?;
    bounded_id(&completion.ending_id, "completed ending id")?;
    bounded_values(&completion.scores, "score")?;
    bounded_evidence(&completion.observed_evidence)?;
    normalize_relationships(completion.relationships.clone())?;
    Ok(())
}

fn normalize_relationships(
    values: BTreeMap<String, f32>,
) -> Result<BTreeMap<String, f32>, RoleplayCampaignError> {
    if values.len() > MAX_SUMMARY_VALUES {
        return invalid_session(format!("relationship values exceed {MAX_SUMMARY_VALUES}"));
    }
    values
        .into_iter()
        .map(|(id, value)| {
            bounded_id(&id, "relationship character id")?;
            if !value.is_finite() || !(-1.0..=1.0).contains(&value) {
                return invalid_session(format!(
                    "relationship `{id}` must be finite and between -1 and 1"
                ));
            }
            Ok((id, value))
        })
        .collect()
}

fn bounded_values(
    values: &BTreeMap<String, f32>,
    label: &str,
) -> Result<BTreeMap<String, f32>, RoleplayCampaignError> {
    if values.len() > MAX_SUMMARY_VALUES {
        return invalid_session(format!("{label} values exceed {MAX_SUMMARY_VALUES}"));
    }
    for (id, value) in values {
        bounded_id(id, &format!("{label} id"))?;
        if !value.is_finite() {
            return invalid_session(format!("{label} `{id}` must be finite"));
        }
    }
    Ok(values.clone())
}

fn bounded_evidence(values: &[String]) -> Result<Vec<String>, RoleplayCampaignError> {
    if values.len() > MAX_SUMMARY_VALUES {
        return invalid_session(format!("observed evidence exceeds {MAX_SUMMARY_VALUES}"));
    }
    let mut seen = HashSet::new();
    for value in values {
        bounded_id(value, "observed evidence id")?;
        if !seen.insert(value.as_str()) {
            return invalid_session(format!("observed evidence repeats `{value}`"));
        }
    }
    Ok(values.to_vec())
}

fn bounded_id(value: &str, label: &str) -> Result<(), RoleplayCampaignError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        return invalid_definition(format!("{label} `{value}` is not a portable id"));
    }
    Ok(())
}

fn bounded_text(value: &str, label: &str, max: usize) -> Result<(), RoleplayCampaignError> {
    if value.trim().is_empty() || value.chars().count() > max {
        return invalid_definition(format!(
            "{label} must contain between 1 and {max} characters"
        ));
    }
    Ok(())
}

fn invalid_definition<T>(message: impl Into<String>) -> Result<T, RoleplayCampaignError> {
    Err(RoleplayCampaignError::InvalidDefinition(message.into()))
}

fn invalid_session<T>(message: impl Into<String>) -> Result<T, RoleplayCampaignError> {
    Err(RoleplayCampaignError::InvalidSession(message.into()))
}

#[cfg(test)]
mod tests;
