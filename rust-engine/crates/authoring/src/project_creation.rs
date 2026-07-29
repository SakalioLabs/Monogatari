//! Transactional creation of an empty, portable Monogatari project.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::project::{canonical_project_root, default_project_config};

const PROJECT_DIRECTORIES: &[&str] = &[
    "assets",
    "campaigns",
    "characters",
    "dialogue",
    "endings",
    "events",
    "knowledge",
    "locales",
    "quality_suites",
    "roleplays",
    "saves",
    "scenes",
    "workflows",
];
const MAX_PROJECT_TITLE_CHARACTERS: usize = 120;
const MAX_DIRECTORY_NAME_CHARACTERS: usize = 80;
static STAGING_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreatedProject {
    pub project_path: String,
    pub project_title: String,
    pub directory_name: String,
}

pub fn create_empty_project(
    parent_directory: &Path,
    directory_name: &str,
    project_title: &str,
) -> Result<CreatedProject, String> {
    let parent_directory = canonical_project_root(parent_directory)?;
    let directory_name = validate_directory_name(directory_name)?;
    let project_title = validate_project_title(project_title)?;
    let destination = parent_directory.join(&directory_name);
    if destination.exists() {
        return Err(format!(
            "A file or directory already exists at `{}`.",
            destination.display()
        ));
    }

    let staging = allocate_staging_path(&parent_directory)?;
    std::fs::create_dir(&staging)
        .map_err(|error| format!("Unable to create project staging directory: {error}"))?;
    let result = build_staged_project(&staging, &project_title).and_then(|_| {
        std::fs::rename(&staging, &destination)
            .map_err(|error| format!("Unable to commit the new project: {error}"))
    });
    if let Err(error) = result {
        let _ = std::fs::remove_dir_all(&staging);
        return Err(error);
    }

    let canonical_destination = destination
        .canonicalize()
        .map_err(|error| format!("Unable to resolve the new project: {error}"))?;
    Ok(CreatedProject {
        project_path: canonical_destination.to_string_lossy().to_string(),
        project_title,
        directory_name,
    })
}

fn build_staged_project(staging: &Path, project_title: &str) -> Result<(), String> {
    for directory in PROJECT_DIRECTORIES {
        std::fs::create_dir(staging.join(directory))
            .map_err(|error| format!("Unable to create `{directory}`: {error}"))?;
    }

    let mut settings = default_project_config();
    settings["render"]["title"] = project_title.into();
    let bytes = serde_json::to_vec_pretty(&settings)
        .map_err(|error| format!("Unable to encode project settings: {error}"))?;
    let settings_path = staging.join("settings.json");
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&settings_path)
        .map_err(|error| format!("Unable to create project settings: {error}"))?;
    file.write_all(&bytes)
        .and_then(|_| file.flush())
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("Unable to write project settings: {error}"))
}

fn validate_project_title(value: &str) -> Result<String, String> {
    let title = value.trim();
    if title.is_empty()
        || title.chars().count() > MAX_PROJECT_TITLE_CHARACTERS
        || title.chars().any(char::is_control)
    {
        return Err("Project title must contain 1 to 120 visible characters.".to_string());
    }
    Ok(title.to_string())
}

fn validate_directory_name(value: &str) -> Result<String, String> {
    let name = value.trim();
    let path = Path::new(name);
    if name.is_empty()
        || name.chars().count() > MAX_DIRECTORY_NAME_CHARACTERS
        || name.starts_with('.')
        || name.ends_with(['.', ' '])
        || name.chars().any(char::is_control)
        || name
            .chars()
            .any(|character| "<>:\"/\\|?*".contains(character))
        || path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
        || is_windows_reserved_name(name)
    {
        return Err(
            "Project directory name must be one portable directory segment of 1 to 80 characters."
                .to_string(),
        );
    }
    Ok(name.to_string())
}

fn is_windows_reserved_name(value: &str) -> bool {
    let stem = value
        .split('.')
        .next()
        .unwrap_or(value)
        .to_ascii_uppercase();
    matches!(
        stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

fn allocate_staging_path(parent: &Path) -> Result<PathBuf, String> {
    for _ in 0..1000 {
        let counter = STAGING_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(
            ".monogatari-project-{}-{counter}.tmp",
            std::process::id()
        ));
        if !path.exists() {
            return Ok(path);
        }
    }
    Err("Unable to allocate a project staging directory.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_project_creation_{label}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn creates_an_empty_project_atomically() {
        let parent = temp_root("create");
        std::fs::create_dir_all(&parent).unwrap();

        let created = create_empty_project(&parent, "first-story", "First Story").unwrap();
        let root = PathBuf::from(&created.project_path);
        assert_eq!(created.project_title, "First Story");
        assert!(root.join("settings.json").is_file());
        for directory in PROJECT_DIRECTORIES {
            assert!(root.join(directory).is_dir(), "{directory}");
        }
        let settings: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("settings.json")).unwrap()).unwrap();
        assert_eq!(settings["render"]["title"], "First Story");
        assert_eq!(settings["ai"]["api"]["api_key"], "");

        std::fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn rejects_collisions_and_non_portable_names_without_partial_output() {
        let parent = temp_root("reject");
        std::fs::create_dir_all(parent.join("existing")).unwrap();

        assert!(create_empty_project(&parent, "existing", "Existing").is_err());
        for name in ["../escape", "CON", ".hidden", "bad.", "a/b"] {
            assert!(
                create_empty_project(&parent, name, "Story").is_err(),
                "{name}"
            );
        }
        assert!(std::fs::read_dir(&parent)
            .unwrap()
            .flatten()
            .all(|entry| !entry
                .file_name()
                .to_string_lossy()
                .starts_with(".monogatari-project-")));

        std::fs::remove_dir_all(parent).unwrap();
    }
}
