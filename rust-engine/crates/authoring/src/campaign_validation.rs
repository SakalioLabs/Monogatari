//! Campaign catalog loading and cross-roleplay route validation.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use llm_game::campaign::RoleplayCampaignDefinition;
use llm_game::scene_roleplay::{RoleplayTarget, SceneRoleplayDefinition};

use crate::scene_roleplay_validation::LoadedSceneRoleplay;

const MAX_CAMPAIGN_FILES: usize = 64;
const MAX_CAMPAIGN_FILE_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone)]
pub struct LoadedRoleplayCampaign {
    pub definition: RoleplayCampaignDefinition,
    pub source_path: String,
    pub absolute_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleplayCampaignReferenceIssue {
    pub code: String,
    pub path: String,
    pub message: String,
}

pub fn load_project_roleplay_campaigns(
    project_root: &Path,
) -> Result<Vec<LoadedRoleplayCampaign>, String> {
    let directory = project_root.join("campaigns");
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let directory_metadata = std::fs::symlink_metadata(&directory)
        .map_err(|error| format!("Failed to inspect `{}`: {error}", directory.display()))?;
    if directory_metadata.file_type().is_symlink() || !directory_metadata.is_dir() {
        return Err(format!(
            "Roleplay campaign catalog must be a regular directory: {}",
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
    if paths.len() > MAX_CAMPAIGN_FILES {
        return Err(format!(
            "Roleplay campaign catalog contains {} JSON files; the limit is {MAX_CAMPAIGN_FILES}.",
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
                "Roleplay campaign document must be a regular file: {}",
                path.display()
            ));
        }
        if metadata.len() > MAX_CAMPAIGN_FILE_BYTES {
            return Err(format!(
                "Roleplay campaign document `{}` is {} bytes; the limit is {MAX_CAMPAIGN_FILE_BYTES} bytes.",
                path.display(),
                metadata.len()
            ));
        }
        let absolute_path = path
            .canonicalize()
            .map_err(|error| format!("Failed to resolve `{}`: {error}", path.display()))?;
        if !absolute_path.starts_with(&canonical_directory) {
            return Err(format!(
                "Roleplay campaign document escapes its catalog: {}",
                path.display()
            ));
        }
        let bytes = std::fs::read(&absolute_path).map_err(|error| {
            format!(
                "Failed to read roleplay campaign `{}`: {error}",
                absolute_path.display()
            )
        })?;
        let definition: RoleplayCampaignDefinition =
            serde_json::from_slice(&bytes).map_err(|error| {
                format!(
                    "Invalid roleplay campaign JSON in `{}`: {error}",
                    absolute_path.display()
                )
            })?;
        definition.validate().map_err(|error| {
            format!(
                "Roleplay campaign `{}` failed validation: {error}",
                absolute_path.display()
            )
        })?;
        if !ids.insert(definition.id.clone()) {
            return Err(format!(
                "Duplicate roleplay campaign id `{}`.",
                definition.id
            ));
        }
        let file_name = absolute_path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| "Roleplay campaign file name is not valid UTF-8.".to_string())?;
        loaded.push(LoadedRoleplayCampaign {
            definition,
            source_path: format!("campaigns/{file_name}"),
            absolute_path,
        });
    }
    loaded.sort_by(|left, right| left.definition.id.cmp(&right.definition.id));
    Ok(loaded)
}

pub fn validate_roleplay_campaign_references(
    campaigns: &[LoadedRoleplayCampaign],
    roleplays: &[LoadedSceneRoleplay],
) -> Vec<RoleplayCampaignReferenceIssue> {
    let roleplay_endings = roleplays
        .iter()
        .map(|loaded| {
            (
                loaded.definition.id.as_str(),
                roleplay_ending_ids(&loaded.definition),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut issues = Vec::new();
    for loaded in campaigns {
        for entry in &loaded.definition.entries {
            let path = format!("{}/{}", loaded.source_path, entry.id);
            let Some(ending_ids) = roleplay_endings.get(entry.roleplay_id.as_str()) else {
                issues.push(reference_issue(
                    "campaign_roleplay_missing",
                    &path,
                    format!(
                        "Campaign entry `{}` references unknown roleplay `{}`.",
                        entry.id, entry.roleplay_id
                    ),
                ));
                continue;
            };
            let routed = entry
                .routes
                .iter()
                .map(|route| route.ending_id.as_str())
                .collect::<HashSet<_>>();
            for route in &entry.routes {
                if !ending_ids.contains(route.ending_id.as_str()) {
                    issues.push(reference_issue(
                        "campaign_ending_not_produced",
                        &path,
                        format!(
                            "Campaign entry `{}` routes ending `{}`, but roleplay `{}` cannot produce it.",
                            entry.id, route.ending_id, entry.roleplay_id
                        ),
                    ));
                }
            }
            let mut missing = ending_ids
                .iter()
                .filter(|ending_id| !routed.contains(**ending_id))
                .copied()
                .collect::<Vec<_>>();
            missing.sort_unstable();
            for ending_id in missing {
                issues.push(reference_issue(
                    "campaign_ending_route_missing",
                    &path,
                    format!(
                        "Campaign entry `{}` has no route for roleplay ending `{ending_id}`.",
                        entry.id
                    ),
                ));
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

pub fn roleplay_ending_ids(definition: &SceneRoleplayDefinition) -> HashSet<&str> {
    let mut endings = HashSet::from([definition.exhaustion_ending_id.as_str()]);
    for node in &definition.nodes {
        for target in node
            .transitions
            .iter()
            .map(|transition| &transition.target)
            .chain(std::iter::once(&node.timeout_target))
        {
            if let RoleplayTarget::Ending { ending_id } = target {
                endings.insert(ending_id);
            }
        }
    }
    endings
}

fn reference_issue(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> RoleplayCampaignReferenceIssue {
    RoleplayCampaignReferenceIssue {
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests;
