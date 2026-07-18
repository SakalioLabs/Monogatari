//! Scene-roleplay catalog loading and cross-project reference validation.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use llm_game::scene_roleplay::{RoleplayTarget, SceneRoleplayDefinition};

const MAX_ROLEPLAY_FILES: usize = 256;
const MAX_ROLEPLAY_FILE_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone)]
pub struct LoadedSceneRoleplay {
    pub definition: SceneRoleplayDefinition,
    pub source_path: String,
    pub absolute_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneRoleplayReferenceIssue {
    pub code: String,
    pub path: String,
    pub message: String,
}

pub fn load_project_scene_roleplays(
    project_root: &Path,
) -> Result<Vec<LoadedSceneRoleplay>, String> {
    let directory = project_root.join("roleplays");
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let directory_metadata = std::fs::symlink_metadata(&directory)
        .map_err(|error| format!("Failed to inspect `{}`: {error}", directory.display()))?;
    if directory_metadata.file_type().is_symlink() || !directory_metadata.is_dir() {
        return Err(format!(
            "Scene roleplay catalog must be a regular directory: {}",
            directory.display()
        ));
    }
    let canonical_directory = directory
        .canonicalize()
        .map_err(|error| format!("Failed to resolve `{}`: {error}", directory.display()))?;
    let mut paths = std::fs::read_dir(&directory)
        .map_err(|error| format!("Failed to read `{}`: {error}", directory.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        })
        .collect::<Vec<_>>();
    paths.sort();
    if paths.len() > MAX_ROLEPLAY_FILES {
        return Err(format!(
            "Scene roleplay catalog contains {} JSON files; the limit is {MAX_ROLEPLAY_FILES}.",
            paths.len()
        ));
    }

    let mut loaded = Vec::with_capacity(paths.len());
    let mut ids = HashSet::new();
    for path in paths {
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|error| format!("Failed to inspect `{}`: {error}", path.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "Scene roleplay document must be a regular file: {}",
                path.display()
            ));
        }
        if metadata.len() > MAX_ROLEPLAY_FILE_BYTES {
            return Err(format!(
                "Scene roleplay document `{}` is {} bytes; the limit is {MAX_ROLEPLAY_FILE_BYTES} bytes.",
                path.display(),
                metadata.len()
            ));
        }
        let absolute_path = path
            .canonicalize()
            .map_err(|error| format!("Failed to resolve `{}`: {error}", path.display()))?;
        if !absolute_path.starts_with(&canonical_directory) {
            return Err(format!(
                "Scene roleplay document escapes its catalog: {}",
                path.display()
            ));
        }
        let bytes = std::fs::read(&absolute_path).map_err(|error| {
            format!(
                "Failed to read scene roleplay `{}`: {error}",
                absolute_path.display()
            )
        })?;
        let definition: SceneRoleplayDefinition =
            serde_json::from_slice(&bytes).map_err(|error| {
                format!(
                    "Invalid scene roleplay JSON in `{}`: {error}",
                    absolute_path.display()
                )
            })?;
        definition.validate().map_err(|error| {
            format!(
                "Scene roleplay `{}` failed validation: {error}",
                absolute_path.display()
            )
        })?;
        if !ids.insert(definition.id.clone()) {
            return Err(format!("Duplicate scene roleplay id `{}`.", definition.id));
        }
        let file_name = absolute_path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| "Scene roleplay file name is not valid UTF-8.".to_string())?;
        loaded.push(LoadedSceneRoleplay {
            definition,
            source_path: format!("roleplays/{file_name}"),
            absolute_path,
        });
    }
    loaded.sort_by(|left, right| left.definition.id.cmp(&right.definition.id));
    Ok(loaded)
}

pub fn validate_scene_roleplay_references(
    roleplays: &[LoadedSceneRoleplay],
    scene_ids: &HashSet<String>,
    character_ids: &HashSet<String>,
    knowledge_ids: &HashSet<String>,
    ending_ids: &HashSet<String>,
) -> Vec<SceneRoleplayReferenceIssue> {
    let mut issues = Vec::new();
    for loaded in roleplays {
        let definition = &loaded.definition;
        if !ending_ids.contains(&definition.exhaustion_ending_id) {
            issues.push(reference_issue(
                "roleplay_exhaustion_ending_missing",
                &loaded.source_path,
                format!(
                    "Scene roleplay `{}` references unknown exhaustion ending `{}`.",
                    definition.id, definition.exhaustion_ending_id
                ),
            ));
        }
        for node in &definition.nodes {
            let node_path = format!("{}/{}", loaded.source_path, node.id);
            if !scene_ids.contains(&node.scene_id) {
                issues.push(reference_issue(
                    "roleplay_scene_missing",
                    &node_path,
                    format!(
                        "Scene roleplay node `{}` references unknown scene `{}`.",
                        node.id, node.scene_id
                    ),
                ));
            }
            for character_id in
                std::iter::once(&node.character_id).chain(node.supporting_character_ids.iter())
            {
                if !character_ids.contains(character_id) {
                    issues.push(reference_issue(
                        "roleplay_character_missing",
                        &node_path,
                        format!(
                            "Scene roleplay node `{}` references unknown character `{character_id}`.",
                            node.id
                        ),
                    ));
                }
            }
            for knowledge_id in &node.knowledge_refs {
                if !knowledge_ids.contains(knowledge_id) {
                    issues.push(reference_issue(
                        "roleplay_knowledge_missing",
                        &node_path,
                        format!(
                            "Scene roleplay node `{}` references unknown knowledge `{knowledge_id}`.",
                            node.id
                        ),
                    ));
                }
            }
            for target in node
                .transitions
                .iter()
                .map(|transition| &transition.target)
                .chain(std::iter::once(&node.timeout_target))
            {
                if let RoleplayTarget::Ending { ending_id } = target {
                    if !ending_ids.contains(ending_id) {
                        issues.push(reference_issue(
                            "roleplay_ending_missing",
                            &node_path,
                            format!(
                                "Scene roleplay node `{}` references unknown ending `{ending_id}`.",
                                node.id
                            ),
                        ));
                    }
                }
            }
        }
    }
    issues.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.code.cmp(&right.code))
            .then_with(|| left.message.cmp(&right.message))
    });
    issues
}

fn reference_issue(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> SceneRoleplayReferenceIssue {
    SceneRoleplayReferenceIssue {
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests;
