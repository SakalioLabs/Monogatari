//! Cross-catalog reference discovery used before destructive authoring changes.

use std::path::Path;

use llm_authoring::dialogue_validation::MAX_DIALOGUE_FILE_BYTES;
use llm_game::characters::Character;
use llm_game::dialogue::DialogueScript;

use crate::commands::endings::load_story_endings;
use crate::commands::workflow::Workflow;
use crate::story_events::{StoryEventAction, StoryEventCatalog};

const MAX_WORKFLOW_REFERENCE_FILES: usize = 512;
const MAX_WORKFLOW_REFERENCE_BYTES: u64 = 1024 * 1024;
const MAX_DIALOGUE_REFERENCE_FILES: usize = 512;
const MAX_CHARACTER_REFERENCE_FILES: usize = 512;
const MAX_CHARACTER_REFERENCE_BYTES: u64 = 2 * 1024 * 1024;

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
    references.extend(dialogue_scene_references(project_root, scene_id)?);
    references.extend(workflow_scene_references(project_root, scene_id)?);
    references.sort();
    references.dedup();
    Ok(references)
}

fn dialogue_scene_references(project_root: &Path, scene_id: &str) -> Result<Vec<String>, String> {
    let dialogue_root = project_root.join("dialogue");
    if !dialogue_root.exists() {
        return Ok(Vec::new());
    }
    let root_metadata = std::fs::symlink_metadata(&dialogue_root).map_err(|error| {
        format!(
            "Failed to inspect dialogue directory `{}`: {error}",
            dialogue_root.display()
        )
    })?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err("Dialogue reference path must be a regular directory.".to_string());
    }
    let canonical_root = dialogue_root.canonicalize().map_err(|error| {
        format!(
            "Failed to resolve dialogue directory `{}`: {error}",
            dialogue_root.display()
        )
    })?;
    let mut paths = std::fs::read_dir(&dialogue_root)
        .map_err(|error| {
            format!(
                "Failed to read dialogue directory `{}`: {error}",
                dialogue_root.display()
            )
        })?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to enumerate dialogue files: {error}"))?;
    paths.retain(|path| {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
    });
    paths.sort();
    if paths.len() > MAX_DIALOGUE_REFERENCE_FILES {
        return Err(format!(
            "Dialogue directory contains {} JSON files; the reference-scan limit is {MAX_DIALOGUE_REFERENCE_FILES}.",
            paths.len()
        ));
    }

    let mut references = Vec::new();
    for path in paths {
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|error| format!("Failed to inspect dialogue `{}`: {error}", path.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "Dialogue must be a regular file: {}",
                path.display()
            ));
        }
        if metadata.len() > MAX_DIALOGUE_FILE_BYTES {
            return Err(format!(
                "Dialogue `{}` is {} bytes; the reference-scan limit is {MAX_DIALOGUE_FILE_BYTES} bytes.",
                path.display(),
                metadata.len()
            ));
        }
        let canonical_path = path
            .canonicalize()
            .map_err(|error| format!("Failed to resolve dialogue `{}`: {error}", path.display()))?;
        if !canonical_path.starts_with(&canonical_root) {
            return Err(format!(
                "Dialogue escapes the project dialogue directory: {}",
                path.display()
            ));
        }
        let source = std::fs::read_to_string(&canonical_path)
            .map_err(|error| format!("Failed to read dialogue `{}`: {error}", path.display()))?;
        let dialogue: DialogueScript = serde_json::from_str(&source)
            .map_err(|error| format!("Invalid dialogue JSON in `{}`: {error}", path.display()))?;
        for (node_id, node) in dialogue.nodes {
            if node.scene_id.as_deref() == Some(scene_id) {
                references.push(format!("dialogue:{}/{}", dialogue.id, node_id));
            }
        }
    }
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

pub fn knowledge_references(
    project_root: &Path,
    knowledge_id: &str,
) -> Result<Vec<String>, String> {
    let character_root = project_root.join("characters");
    if !character_root.exists() {
        return Ok(Vec::new());
    }
    let root_metadata = std::fs::symlink_metadata(&character_root).map_err(|error| {
        format!(
            "Failed to inspect character directory `{}`: {error}",
            character_root.display()
        )
    })?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err("Character reference path must be a regular directory.".to_string());
    }

    let mut paths = std::fs::read_dir(&character_root)
        .map_err(|error| {
            format!(
                "Failed to read character directory `{}`: {error}",
                character_root.display()
            )
        })?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to enumerate character files: {error}"))?;
    paths.retain(|path| {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
    });
    paths.sort();
    if paths.len() > MAX_CHARACTER_REFERENCE_FILES {
        return Err(format!(
            "Character directory contains {} JSON files; the reference-scan limit is {MAX_CHARACTER_REFERENCE_FILES}.",
            paths.len()
        ));
    }

    let mut references = Vec::new();
    for path in paths {
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
            format!("Failed to inspect character `{}`: {error}", path.display())
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "Character must be a regular file: {}",
                path.display()
            ));
        }
        if metadata.len() > MAX_CHARACTER_REFERENCE_BYTES {
            return Err(format!(
                "Character `{}` is {} bytes; the reference-scan limit is {MAX_CHARACTER_REFERENCE_BYTES} bytes.",
                path.display(),
                metadata.len()
            ));
        }
        let source = std::fs::read_to_string(&path)
            .map_err(|error| format!("Failed to read character `{}`: {error}", path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&source)
            .map_err(|error| format!("Invalid character JSON in `{}`: {error}", path.display()))?;
        let values = match value {
            serde_json::Value::Object(_) => vec![value],
            serde_json::Value::Array(values) => values,
            _ => {
                return Err(format!(
                    "Character file `{}` must contain an object or array.",
                    path.display()
                ))
            }
        };
        for value in values {
            let character: Character = serde_json::from_value(value)
                .map_err(|error| format!("Invalid character in `{}`: {error}", path.display()))?;
            if character
                .knowledge_refs
                .iter()
                .any(|reference| reference.trim() == knowledge_id)
            {
                references.push(format!("character:{}", character.id));
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

    fn temp_root(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari-content-references-{label}-{}-{}",
            std::process::id(),
            NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn knowledge_references_find_character_pins() {
        let root = temp_root("knowledge");
        let characters = root.join("characters");
        std::fs::create_dir_all(&characters).unwrap();
        std::fs::write(
            characters.join("cast.json"),
            r#"[{"id":"aoi","name":"Aoi","knowledge_refs":["herbal_lore"]},{"id":"sora","name":"Sora","knowledge_refs":["star_lore"]}]"#,
        )
        .unwrap();

        let references = knowledge_references(&root, "herbal_lore").unwrap();
        let _ = std::fs::remove_dir_all(&root);

        assert_eq!(references, vec!["character:aoi"]);
    }

    #[test]
    fn scene_references_find_dialogue_node_transitions() {
        let root = temp_root("dialogue-scene");
        let dialogues = root.join("dialogue");
        std::fs::create_dir_all(&dialogues).unwrap();
        std::fs::write(
            dialogues.join("route.json"),
            r#"{
              "id":"archive_route",
              "title":"Archive Route",
              "start_node_id":"arrival",
              "nodes":{
                "arrival":{"text":"Arrival","scene_id":"archive","next_node_id":"end"},
                "end":{"text":"End","is_ending":true}
              }
            }"#,
        )
        .unwrap();

        let references = scene_references(&root, "archive").unwrap();
        let _ = std::fs::remove_dir_all(&root);

        assert_eq!(references, vec!["dialogue:archive_route/arrival"]);
    }
}
