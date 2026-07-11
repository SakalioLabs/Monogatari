//! Portable project path resolution shared by every authoring transport.

use std::path::{Component, Path, PathBuf};

/// Resolve a frontend content-directory request beneath its canonical catalog root.
pub fn project_content_dir(
    project_root: &Path,
    requested_dir: &str,
    canonical_dir: &str,
) -> Result<PathBuf, String> {
    let mut segments = normalize_content_relative_path(requested_dir)?;
    if segments.first().map(String::as_str) == Some(canonical_dir) {
        segments.remove(0);
    }

    let root = project_root.join(canonical_dir);
    let path = segments
        .iter()
        .fold(root.clone(), |path, segment| path.join(segment));

    if !path.starts_with(&root) {
        return Err(
            "Content directory path must stay inside the active project content root.".to_string(),
        );
    }

    Ok(path)
}

/// Resolve a persisted portable path beneath the project root.
pub fn resolve_project_relative(project_root: &Path, relative: &str) -> Result<PathBuf, String> {
    if relative.trim().is_empty() || relative.chars().any(char::is_control) {
        return Err(
            "Project paths must be non-empty and cannot contain control characters.".to_string(),
        );
    }
    if relative.contains('\\') {
        return Err("Project paths must use portable forward slashes.".to_string());
    }
    if relative.contains(':') {
        return Err("Project paths cannot contain drive prefixes or URI schemes.".to_string());
    }

    let segments = relative.split('/').collect::<Vec<_>>();
    if segments
        .iter()
        .any(|segment| segment.is_empty() || *segment == "." || *segment == "..")
    {
        return Err(
            "Project paths cannot contain empty, current, or parent directory segments."
                .to_string(),
        );
    }

    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return Err("Project paths must stay inside the project root.".to_string());
    }

    Ok(segments
        .into_iter()
        .fold(project_root.to_path_buf(), |path, segment| {
            path.join(segment)
        }))
}

fn normalize_content_relative_path(requested_dir: &str) -> Result<Vec<String>, String> {
    let normalized = requested_dir.trim().replace('\\', "/");
    if normalized.is_empty() || normalized.chars().any(char::is_control) {
        return Err(
            "Content paths must be non-empty and cannot contain control characters.".to_string(),
        );
    }
    if normalized.contains(':') {
        return Err("Content paths cannot contain drive prefixes or URI schemes.".to_string());
    }

    let segments = normalized.split('/').collect::<Vec<_>>();
    if segments
        .iter()
        .any(|segment| segment.is_empty() || *segment == "." || *segment == "..")
    {
        return Err(
            "Content paths cannot contain empty, current, or parent directory segments."
                .to_string(),
        );
    }

    let path = Path::new(&normalized);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return Err(
            "Content paths must be relative to the active project content root.".to_string(),
        );
    }

    Ok(segments.into_iter().map(str::to_string).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_dirs_resolve_canonical_and_nested_project_paths() {
        let root = PathBuf::from("project-data");

        assert_eq!(
            project_content_dir(&root, "dialogue", "dialogue").unwrap(),
            root.join("dialogue")
        );
        assert_eq!(
            project_content_dir(&root, "dialogue/chapter1", "dialogue").unwrap(),
            root.join("dialogue").join("chapter1")
        );
        assert_eq!(
            project_content_dir(&root, "chapter1", "dialogue").unwrap(),
            root.join("dialogue").join("chapter1")
        );
        assert_eq!(
            project_content_dir(&root, "characters\\sakura", "characters").unwrap(),
            root.join("characters").join("sakura")
        );
    }

    #[test]
    fn content_dirs_reject_escape_attempts() {
        let root = PathBuf::from("project-data");
        for path in [
            "",
            "../secrets",
            "dialogue/../secrets",
            "dialogue//chapter1",
            "dialogue/./chapter1",
            "C:/Users/example/dialogue",
            "https://example.test/dialogue",
            "/tmp/dialogue",
            "dialogue:chapter1",
        ] {
            assert!(
                project_content_dir(&root, path, "dialogue").is_err(),
                "{path} should be rejected"
            );
        }
    }

    #[test]
    fn persisted_project_paths_are_strictly_portable() {
        let root = PathBuf::from("project-data");
        assert_eq!(
            resolve_project_relative(&root, "assets/backgrounds").unwrap(),
            root.join("assets").join("backgrounds")
        );

        for path in [
            "",
            ".",
            "../assets",
            "assets/../private",
            "assets//backgrounds",
            "assets\\backgrounds",
            "C:/assets",
            "https://example.test/assets",
            "/tmp/assets",
        ] {
            assert!(
                resolve_project_relative(&root, path).is_err(),
                "{path} should be rejected"
            );
        }
    }
}
