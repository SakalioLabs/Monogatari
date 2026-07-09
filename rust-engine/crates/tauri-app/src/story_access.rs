//! Story content access derived from event unlock actions and persisted progress.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::story_events::{StoryEventAction, StoryEventCatalog};
use crate::story_progress::StoryProgressState;

pub const STORY_CONTENT_ACCESS_SCHEMA_V1: &str = "monogatari-story-content-access/v1";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum StoryContentKind {
    Scene,
    Dialogue,
    Ending,
}

impl StoryContentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Scene => "scene",
            Self::Dialogue => "dialogue",
            Self::Ending => "ending",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StoryContentAccessEntry {
    pub content_type: StoryContentKind,
    pub content_id: String,
    pub gated: bool,
    pub unlocked: bool,
    pub unlock_event_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StoryContentAccessSnapshot {
    pub schema: String,
    pub catalog_fingerprint: String,
    pub progress_fingerprint: String,
    pub gated_content_count: usize,
    pub unlocked_gated_content_count: usize,
    pub locked_content_count: usize,
    pub entries: Vec<StoryContentAccessEntry>,
}

pub fn story_content_access(
    catalog: &StoryEventCatalog,
    progress: &StoryProgressState,
    content_type: StoryContentKind,
    content_id: &str,
) -> StoryContentAccessEntry {
    let unlock_event_ids: Vec<String> = unlock_sources(catalog)
        .remove(&(content_type, content_id.to_string()))
        .map(BTreeSet::into_iter)
        .map(Iterator::collect)
        .unwrap_or_default();
    let gated = !unlock_event_ids.is_empty();

    StoryContentAccessEntry {
        content_type,
        content_id: content_id.to_string(),
        gated,
        unlocked: !gated || progress_contains(progress, content_type, content_id),
        unlock_event_ids,
    }
}

pub fn ensure_story_content_access(
    catalog: &StoryEventCatalog,
    progress: &StoryProgressState,
    content_type: StoryContentKind,
    content_id: &str,
) -> Result<StoryContentAccessEntry, String> {
    let access = story_content_access(catalog, progress, content_type, content_id);
    if access.unlocked {
        return Ok(access);
    }

    Err(format!(
        "Story {} `{}` is locked; unlock it through event(s): {}.",
        content_type.as_str(),
        content_id,
        access.unlock_event_ids.join(", ")
    ))
}

pub fn story_content_access_snapshot(
    catalog: &StoryEventCatalog,
    progress: &StoryProgressState,
) -> StoryContentAccessSnapshot {
    let mut sources = unlock_sources(catalog);
    add_progress_entries(
        &mut sources,
        StoryContentKind::Scene,
        &progress.unlocked_scene_ids,
    );
    add_progress_entries(
        &mut sources,
        StoryContentKind::Dialogue,
        &progress.unlocked_dialogue_ids,
    );
    add_progress_entries(
        &mut sources,
        StoryContentKind::Ending,
        &progress.unlocked_ending_ids,
    );

    let entries = sources
        .into_iter()
        .map(|((content_type, content_id), unlock_event_ids)| {
            let gated = !unlock_event_ids.is_empty();
            StoryContentAccessEntry {
                content_type,
                unlocked: !gated || progress_contains(progress, content_type, &content_id),
                content_id,
                gated,
                unlock_event_ids: unlock_event_ids.into_iter().collect(),
            }
        })
        .collect::<Vec<_>>();
    let gated_content_count = entries.iter().filter(|entry| entry.gated).count();
    let unlocked_gated_content_count = entries
        .iter()
        .filter(|entry| entry.gated && entry.unlocked)
        .count();

    StoryContentAccessSnapshot {
        schema: STORY_CONTENT_ACCESS_SCHEMA_V1.to_string(),
        catalog_fingerprint: catalog.catalog_fingerprint().to_string(),
        progress_fingerprint: progress.progress_fingerprint(),
        gated_content_count,
        unlocked_gated_content_count,
        locked_content_count: gated_content_count.saturating_sub(unlocked_gated_content_count),
        entries,
    }
}

fn unlock_sources(
    catalog: &StoryEventCatalog,
) -> BTreeMap<(StoryContentKind, String), BTreeSet<String>> {
    let mut sources = BTreeMap::new();
    for definition in catalog.definitions() {
        for action in &definition.actions {
            let target = match action {
                StoryEventAction::UnlockScene { scene_id } => {
                    Some((StoryContentKind::Scene, scene_id))
                }
                StoryEventAction::UnlockDialogue { dialogue_id } => {
                    Some((StoryContentKind::Dialogue, dialogue_id))
                }
                StoryEventAction::UnlockEnding { ending_id } => {
                    Some((StoryContentKind::Ending, ending_id))
                }
                StoryEventAction::SetFlag { .. } => None,
            };
            let Some((content_type, content_id)) = target else {
                continue;
            };
            sources
                .entry((content_type, content_id.clone()))
                .or_insert_with(BTreeSet::new)
                .insert(definition.event_id.clone());
        }
    }
    sources
}

fn add_progress_entries(
    sources: &mut BTreeMap<(StoryContentKind, String), BTreeSet<String>>,
    content_type: StoryContentKind,
    content_ids: &BTreeSet<String>,
) {
    for content_id in content_ids {
        sources
            .entry((content_type, content_id.clone()))
            .or_default();
    }
}

fn progress_contains(
    progress: &StoryProgressState,
    content_type: StoryContentKind,
    content_id: &str,
) -> bool {
    match content_type {
        StoryContentKind::Scene => progress.unlocked_scene_ids.contains(content_id),
        StoryContentKind::Dialogue => progress.unlocked_dialogue_ids.contains(content_id),
        StoryContentKind::Ending => progress.unlocked_ending_ids.contains(content_id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> StoryEventCatalog {
        StoryEventCatalog::default()
    }

    #[test]
    fn content_not_referenced_by_an_unlock_action_is_open() {
        let catalog = catalog();
        let progress = StoryProgressState::default();

        let access = story_content_access(
            &catalog,
            &progress,
            StoryContentKind::Dialogue,
            "sakura_park_walk",
        );

        assert!(!access.gated);
        assert!(access.unlocked);
        assert!(access.unlock_event_ids.is_empty());
    }

    #[test]
    fn referenced_content_stays_locked_until_progress_contains_it() {
        let catalog = catalog();
        let mut progress = StoryProgressState::default();

        let locked = story_content_access(
            &catalog,
            &progress,
            StoryContentKind::Scene,
            "festival_night",
        );
        assert!(locked.gated);
        assert!(!locked.unlocked);
        assert_eq!(locked.unlock_event_ids, vec!["first_friend"]);
        assert!(ensure_story_content_access(
            &catalog,
            &progress,
            StoryContentKind::Scene,
            "festival_night"
        )
        .is_err());

        progress
            .unlocked_scene_ids
            .insert("festival_night".to_string());
        assert!(ensure_story_content_access(
            &catalog,
            &progress,
            StoryContentKind::Scene,
            "festival_night"
        )
        .is_ok());
    }

    #[test]
    fn snapshot_is_sorted_and_counts_gated_unlocks() {
        let catalog = catalog();
        let mut progress = StoryProgressState::default();
        progress
            .unlocked_dialogue_ids
            .insert("festival_preparations".to_string());
        progress
            .unlocked_scene_ids
            .insert("orphan_from_legacy_save".to_string());

        let snapshot = story_content_access_snapshot(&catalog, &progress);

        assert_eq!(snapshot.schema, STORY_CONTENT_ACCESS_SCHEMA_V1);
        assert!(snapshot.gated_content_count > 0);
        assert_eq!(snapshot.unlocked_gated_content_count, 1);
        assert_eq!(
            snapshot.locked_content_count,
            snapshot.gated_content_count - 1
        );
        assert!(snapshot.entries.windows(2).all(|pair| {
            (&pair[0].content_type, &pair[0].content_id)
                <= (&pair[1].content_type, &pair[1].content_id)
        }));
        let orphan = snapshot
            .entries
            .iter()
            .find(|entry| entry.content_id == "orphan_from_legacy_save")
            .unwrap();
        assert!(!orphan.gated);
        assert!(orphan.unlocked);
    }
}
