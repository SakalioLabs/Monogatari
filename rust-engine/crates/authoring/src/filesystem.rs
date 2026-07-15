//! Atomic filesystem transactions for project content authoring.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

static CONTENT_STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct StagedContentMutation {
    target_path: PathBuf,
    backup_path: PathBuf,
    had_target: bool,
}

impl StagedContentMutation {
    pub async fn commit(self) -> Result<(), String> {
        if self.backup_path.exists() {
            tokio::fs::remove_file(&self.backup_path)
                .await
                .map_err(|error| {
                    format!(
                        "Failed to remove content backup `{}`: {error}",
                        self.backup_path.display()
                    )
                })?;
        }
        Ok(())
    }

    pub async fn rollback(&self) -> Result<(), String> {
        if self.target_path.exists() {
            tokio::fs::remove_file(&self.target_path)
                .await
                .map_err(|error| {
                    format!(
                        "Failed to remove rejected content `{}`: {error}",
                        self.target_path.display()
                    )
                })?;
        }
        if self.had_target {
            tokio::fs::rename(&self.backup_path, &self.target_path)
                .await
                .map_err(|error| {
                    format!(
                        "Failed to restore content `{}`: {error}",
                        self.target_path.display()
                    )
                })?;
        }
        Ok(())
    }
}

pub async fn stage_json_replacement(
    target_path: &Path,
    content: &[u8],
    max_bytes: u64,
    label: &str,
) -> Result<StagedContentMutation, String> {
    if content.len() as u64 > max_bytes {
        return Err(format!(
            "{label} is {} bytes; the limit is {max_bytes} bytes.",
            content.len()
        ));
    }
    ensure_portable_replacement_target(target_path, label)?;
    let (temp_path, backup_path) = stage_paths(target_path)?;
    let mut file = tokio::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp_path)
        .await
        .map_err(|error| format!("Failed to stage {label} `{}`: {error}", temp_path.display()))?;
    if let Err(error) = file.write_all(content).await {
        drop(file);
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(format!(
            "Failed to write staged {label} `{}`: {error}",
            temp_path.display()
        ));
    }
    if let Err(error) = file.flush().await {
        drop(file);
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(format!(
            "Failed to flush staged {label} `{}`: {error}",
            temp_path.display()
        ));
    }
    if let Err(error) = file.sync_all().await {
        drop(file);
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(format!(
            "Failed to sync staged {label} `{}`: {error}",
            temp_path.display()
        ));
    }
    drop(file);

    let had_target = target_path.exists();
    if had_target {
        let metadata = std::fs::symlink_metadata(target_path).map_err(|error| {
            format!(
                "Failed to inspect existing {label} `{}`: {error}",
                target_path.display()
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(format!(
                "Existing {label} must be a regular file: {}",
                target_path.display()
            ));
        }
        if let Err(error) = tokio::fs::rename(target_path, &backup_path).await {
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(format!(
                "Failed to back up {label} `{}`: {error}",
                target_path.display()
            ));
        }
    }
    if let Err(error) = tokio::fs::rename(&temp_path, target_path).await {
        if had_target {
            let _ = tokio::fs::rename(&backup_path, target_path).await;
        }
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(format!(
            "Failed to replace {label} `{}`: {error}",
            target_path.display()
        ));
    }

    Ok(StagedContentMutation {
        target_path: target_path.to_path_buf(),
        backup_path,
        had_target,
    })
}

pub async fn stage_json_deletion(
    target_path: &Path,
    label: &str,
) -> Result<StagedContentMutation, String> {
    let metadata = std::fs::symlink_metadata(target_path).map_err(|error| {
        format!(
            "Failed to inspect {label} `{}`: {error}",
            target_path.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "{label} must be a regular file: {}",
            target_path.display()
        ));
    }
    let (_, backup_path) = stage_paths(target_path)?;
    tokio::fs::rename(target_path, &backup_path)
        .await
        .map_err(|error| {
            format!(
                "Failed to stage {label} `{}` for deletion: {error}",
                target_path.display()
            )
        })?;
    Ok(StagedContentMutation {
        target_path: target_path.to_path_buf(),
        backup_path,
        had_target: true,
    })
}

pub async fn ensure_regular_project_directory(
    project_root: &Path,
    directory_name: &str,
    label: &str,
) -> Result<PathBuf, String> {
    let directory = project_root.join(directory_name);
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| {
            format!(
                "Failed to create {label} directory `{}`: {error}",
                directory.display()
            )
        })?;
    let metadata = std::fs::symlink_metadata(&directory).map_err(|error| {
        format!(
            "Failed to inspect {label} directory `{}`: {error}",
            directory.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "{label} path must be a regular directory: {}",
            directory.display()
        ));
    }
    directory.canonicalize().map_err(|error| {
        format!(
            "Failed to resolve {label} directory `{}`: {error}",
            directory.display()
        )
    })
}

pub fn sha256_json(value: &Value) -> String {
    let encoded = serde_json::to_vec(value).expect("content fingerprint payload should serialize");
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    format!("{:x}", hasher.finalize())
}

pub fn source_label(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn ensure_portable_replacement_target(target_path: &Path, label: &str) -> Result<(), String> {
    let parent = target_path
        .parent()
        .ok_or_else(|| "Content target has no parent directory.".to_string())?;
    let target_name = target_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Content target filename is not valid UTF-8.".to_string())?;
    let entries = std::fs::read_dir(parent).map_err(|error| {
        format!(
            "Failed to inspect {label} target directory `{}`: {error}",
            parent.display()
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "Failed to inspect {label} target directory entry `{}`: {error}",
                parent.display()
            )
        })?;
        let Some(existing_name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if existing_name.eq_ignore_ascii_case(target_name) && existing_name != target_name {
            return Err(format!(
                "{label} target `{}` collides with existing path `{}` by ASCII case; use the existing exact filename or choose a distinct portable filename.",
                target_path.display(),
                entry.path().display()
            ));
        }
    }
    Ok(())
}

fn stage_paths(target_path: &Path) -> Result<(PathBuf, PathBuf), String> {
    let parent = target_path
        .parent()
        .ok_or_else(|| "Content target has no parent directory.".to_string())?;
    let file_name = target_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Content target filename is not valid UTF-8.".to_string())?;
    let nonce = CONTENT_STAGE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let stem = format!(".{file_name}.{}.{}", std::process::id(), nonce);
    Ok((
        parent.join(format!("{stem}.tmp")),
        parent.join(format!("{stem}.bak")),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monogatari_content_authoring_{label}_{}_{}",
            std::process::id(),
            CONTENT_STAGE_COUNTER.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[tokio::test]
    async fn replacements_commit_or_restore_the_previous_document() {
        let root = temp_root("replace");
        std::fs::create_dir_all(&root).unwrap();
        let target = root.join("content.json");
        std::fs::write(&target, b"before").unwrap();

        let staged = stage_json_replacement(&target, b"after", 64, "test content")
            .await
            .unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"after");
        staged.rollback().await.unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"before");

        let staged = stage_json_replacement(&target, b"final", 64, "test content")
            .await
            .unwrap();
        staged.commit().await.unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"final");
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn replacements_reject_portable_case_aliases_before_mutation() {
        let root = temp_root("case-alias");
        std::fs::create_dir_all(&root).unwrap();
        let existing = root.join("Scene.json");
        std::fs::write(&existing, b"before").unwrap();

        let error = stage_json_replacement(&root.join("scene.json"), b"after", 64, "test content")
            .await
            .err()
            .expect("portable case aliases must be rejected");

        assert!(error.contains("collides with existing path"), "{error}");
        assert!(error.contains("by ASCII case"), "{error}");
        assert_eq!(std::fs::read(&existing).unwrap(), b"before");
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn deletions_can_be_rolled_back_without_staged_files() {
        let root = temp_root("delete");
        std::fs::create_dir_all(&root).unwrap();
        let target = root.join("content.json");
        std::fs::write(&target, b"before").unwrap();

        let staged = stage_json_deletion(&target, "test content").await.unwrap();
        assert!(!target.exists());
        staged.rollback().await.unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"before");
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }
}
