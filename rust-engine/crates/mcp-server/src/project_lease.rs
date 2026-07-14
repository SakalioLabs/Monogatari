//! Cross-process project leases that never mutate the authored project tree.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

const LEASE_NAMESPACE: &str = "monogatari-mcp-project-leases-v1";

#[derive(Debug)]
pub(crate) struct ProjectLease {
    _file: std::fs::File,
}

impl ProjectLease {
    pub(crate) fn acquire(project_root: &Path, allow_write: bool) -> Result<Self, String> {
        let lock_path = project_lease_path(project_root)?;
        if let Ok(metadata) = std::fs::symlink_metadata(&lock_path) {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err("MCP project lease path must be a regular file.".to_string());
            }
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|_| "Unable to open the MCP project lease.".to_string())?;
        let metadata = std::fs::symlink_metadata(&lock_path)
            .map_err(|_| "Unable to inspect the MCP project lease.".to_string())?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err("MCP project lease path must be a regular file.".to_string());
        }

        if allow_write {
            std::fs::File::try_lock(&file).map_err(|_| {
                "Another MCP server already holds this project root; stop it before enabling writes."
                    .to_string()
            })?;
        } else {
            std::fs::File::try_lock_shared(&file).map_err(|_| {
                "A write-enabled MCP server already owns this project root.".to_string()
            })?;
        }
        Ok(Self { _file: file })
    }
}

fn project_lease_path(project_root: &Path) -> Result<PathBuf, String> {
    let directory = lease_directory()?;
    let project_fingerprint = format!(
        "{:x}",
        Sha256::digest(project_root.as_os_str().as_encoded_bytes())
    );
    Ok(directory.join(format!("{project_fingerprint}.lock")))
}

fn lease_directory() -> Result<PathBuf, String> {
    let directory = std::env::temp_dir().join(LEASE_NAMESPACE);
    std::fs::create_dir_all(&directory)
        .map_err(|_| "Unable to create the MCP project lease directory.".to_string())?;
    let metadata = std::fs::symlink_metadata(&directory)
        .map_err(|_| "Unable to inspect the MCP project lease directory.".to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("MCP project lease directory must be a regular directory.".to_string());
    }
    directory
        .canonicalize()
        .map_err(|_| "Unable to resolve the MCP project lease directory.".to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

    fn temp_project(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "monogatari-mcp-lease-{label}-{}-{}",
            std::process::id(),
            NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("settings.json"), b"{}\n").unwrap();
        root.canonicalize().unwrap()
    }

    fn project_entries(root: &Path) -> Vec<String> {
        let mut entries = std::fs::read_dir(root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        entries.sort();
        entries
    }

    #[test]
    fn lease_paths_are_stable_private_and_project_specific() {
        let first = temp_project("path-a");
        let second = temp_project("path-b");
        let first_path = project_lease_path(&first).unwrap();
        let repeated_path = project_lease_path(&first).unwrap();
        let second_path = project_lease_path(&second).unwrap();

        assert_eq!(first_path, repeated_path);
        assert_ne!(first_path, second_path);
        assert!(!first_path.starts_with(&first));
        let file_name = first_path.file_name().unwrap().to_string_lossy();
        let fingerprint = file_name.strip_suffix(".lock").unwrap();
        assert_eq!(fingerprint.len(), 64);
        assert!(fingerprint
            .chars()
            .all(|character| character.is_ascii_hexdigit()));

        std::fs::remove_dir_all(first).unwrap();
        std::fs::remove_dir_all(second).unwrap();
    }

    #[test]
    fn leases_coordinate_access_without_mutating_the_project() {
        let root = temp_project("coordination");
        let before = project_entries(&root);

        let first_reader = ProjectLease::acquire(&root, false).unwrap();
        let second_reader = ProjectLease::acquire(&root, false).unwrap();
        assert!(ProjectLease::acquire(&root, true).is_err());
        assert_eq!(project_entries(&root), before);
        drop(first_reader);
        drop(second_reader);

        let writer = ProjectLease::acquire(&root, true).unwrap();
        assert!(ProjectLease::acquire(&root, true).is_err());
        assert!(ProjectLease::acquire(&root, false).is_err());
        assert_eq!(project_entries(&root), before);
        drop(writer);

        std::fs::remove_file(project_lease_path(&root).unwrap()).unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
}
