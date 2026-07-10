//! Cross-catalog reference discovery used before destructive authoring changes.

use std::path::Path;

use crate::commands::endings::load_story_endings;
use crate::commands::workflow::Workflow;
use crate::story_events::{StoryEventAction, StoryEventCatalog};

const MAX_WORKFLOW_REFERENCE_FILES: usize = 512;
const MAX_WORKFLOW_REFERENCE_BYTES: u64 = 1024 * 1024;

pub fn scene_references(project_root: &Path, scene_id: &str) -> Result<Vec<String>, String> {
    let mut references = event_references(
        project_root,
        |action| matches!(action, StoryEventAction::UnlockScene { scene_id: target } if target == scene_id),
    )?;
    for ending in load_story_endings(project_root)? {
        if ending.scene_id == scene_id {
            references.push(format!("ending:{}", ending.id));
        }
    }
    references.extend(workflow_scene_references(project_root, scene_id)?);
    references.sort();
    references.dedup();
    Ok(references)
}

pub fn dialogue_references(project_root: &Path, dialogue_id: &str) -> Result<Vec<String>, String> {
    let mut references = event_references(
        project_root,
        |action| matches!(action, StoryEventAction::UnlockDialogue { dialogue_id: target } if target == dialogue_id),
    )?;
    for ending in load_story_endings(project_root)? {
        if ending.dialogue_id == dialogue_id {
            references.push(format!("ending:{}", ending.id));
        }
    }
    references.sort();
    references.dedup();
    Ok(references)
}

fn event_references(
    project_root: &Path,
    matches_action: impl Fn(&StoryEventAction) -> bool,
) -> Result<Vec<String>, String> {
    let catalog = StoryEventCatalog::load_from_project_root(project_root)?;
    Ok(catalog
        .definitions()
        .iter()
        .filter(|definition| definition.actions.iter().any(&matches_action))
        .map(|definition| format!("event:{}", definition.event_id))
        .collect())
}

fn workflow_scene_references(project_root: &Path, scene_id: &str) -> Result<Vec<String>, String> {
    let workflow_root = project_root.join("workflows");
    if !workflow_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths = std::fs::read_dir(&workflow_root)
        .map_err(|error| {
            format!(
                "Failed to read workflow directory `{}`: {error}",
                workflow_root.display()
            )
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        })
        .collect::<Vec<_>>();
    paths.sort();
    if paths.len() > MAX_WORKFLOW_REFERENCE_FILES {
        return Err(format!(
            "Workflow directory contains {} JSON files; the reference-scan limit is {MAX_WORKFLOW_REFERENCE_FILES}.",
            paths.len()
        ));
    }

    let mut references = Vec::new();
    for path in paths {
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|error| format!("Failed to inspect workflow `{}`: {error}", path.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "Workflow must be a regular file: {}",
                path.display()
            ));
        }
        if metadata.len() > MAX_WORKFLOW_REFERENCE_BYTES {
            return Err(format!(
                "Workflow `{}` is {} bytes; the reference-scan limit is {MAX_WORKFLOW_REFERENCE_BYTES} bytes.",
                path.display(),
                metadata.len()
            ));
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|error| format!("Failed to read workflow `{}`: {error}", path.display()))?;
        let workflow: Workflow = serde_json::from_str(&content)
            .map_err(|error| format!("Invalid workflow JSON in `{}`: {error}", path.display()))?;
        for node in workflow.nodes {
            if node.node_type == "scene_change"
                && node.config.get("scene_id").and_then(|value| value.as_str()) == Some(scene_id)
            {
                references.push(format!("workflow:{}/{}", workflow.id, node.id));
            }
        }
    }
    Ok(references)
}
