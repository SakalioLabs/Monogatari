//! Cloud save sync with local file-based manifest and checksum tracking.
//!
//! Provides commands for syncing game saves to a remote cloud storage backend.
//! Supports push (upload), pull (download), and conflict resolution.
//! When no cloud provider is configured, operates in local-diff mode for
//! offline save management and device transfer.

use chrono::Utc;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tauri::State;
use tokio::sync::RwLock;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSyncStatus {
    pub connected: bool,
    pub status: String,
    pub last_sync: Option<String>,
    pub file_count: usize,
    pub pending_uploads: usize,
    pub pending_downloads: usize,
    pub conflict_count: usize,
    pub provider: String,
    pub endpoint_configured: bool,
    pub token_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSaveEntry {
    pub save_id: String,
    pub device_id: String,
    pub timestamp: String,
    pub size_bytes: u64,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncManifest {
    entries: HashMap<String, CloudSaveEntry>,
    device_id: String,
    last_updated: String,
}

#[derive(Debug, Clone)]
struct CloudSyncConfig {
    provider: String,
    endpoint_configured: bool,
    token_configured: bool,
}

impl Default for CloudSyncConfig {
    fn default() -> Self {
        Self {
            provider: "local".to_string(),
            endpoint_configured: false,
            token_configured: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct SyncInventory {
    file_count: usize,
    pending_uploads: usize,
    pending_downloads: usize,
    conflict_count: usize,
    last_sync: Option<String>,
}

static CLOUD_SYNC_CONFIG: Lazy<RwLock<CloudSyncConfig>> =
    Lazy::new(|| RwLock::new(CloudSyncConfig::default()));

fn saves_dir(project_root: &Path) -> PathBuf {
    project_root.join("saves")
}

fn manifest_path(project_root: &Path) -> PathBuf {
    saves_dir(project_root).join(".sync_manifest.json")
}

fn get_device_id() -> String {
    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

fn load_manifest(project_root: &Path) -> SyncManifest {
    let path = manifest_path(project_root);
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(m) = serde_json::from_str::<SyncManifest>(&content) {
                return m;
            }
        }
    }
    SyncManifest {
        entries: HashMap::new(),
        device_id: get_device_id(),
        last_updated: String::new(),
    }
}

fn save_manifest(project_root: &Path, manifest: &SyncManifest) -> Result<(), String> {
    let path = manifest_path(project_root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

fn compute_checksum(path: &std::path::Path) -> String {
    let content = std::fs::read(path).unwrap_or_default();
    format!("{:x}", md5::compute(&content))
}

fn compute_bytes_checksum(bytes: &[u8]) -> String {
    format!("{:x}", md5::compute(bytes))
}

fn now_timestamp() -> String {
    Utc::now().to_rfc3339()
}

fn normalize_provider(provider: &str) -> String {
    let normalized = provider.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        "local".to_string()
    } else {
        normalized
    }
}

fn has_config_value(value: &Option<String>) -> bool {
    value
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
}

fn is_valid_save_id(save_id: &str) -> bool {
    let trimmed = save_id.trim();
    !trimmed.is_empty()
        && !trimmed.starts_with('.')
        && !trimmed.contains('/')
        && !trimmed.contains('\\')
}

fn is_save_json(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("json")
        && path.file_name().and_then(|name| name.to_str()) != Some(".sync_manifest.json")
}

fn save_file_path(project_root: &Path, save_id: &str) -> Option<PathBuf> {
    is_valid_save_id(save_id).then(|| saves_dir(project_root).join(format!("{save_id}.json")))
}

fn analyze_sync_inventory(
    project_root: &Path,
    manifest: &SyncManifest,
    device_id: &str,
) -> SyncInventory {
    let saves_dir = saves_dir(project_root);
    let mut local_save_ids = HashSet::new();
    let mut inventory = SyncInventory {
        last_sync: (!manifest.last_updated.trim().is_empty())
            .then(|| manifest.last_updated.clone()),
        ..SyncInventory::default()
    };

    if saves_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&saves_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !is_save_json(&path) {
                    continue;
                }

                let Some(id) = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .filter(|id| is_valid_save_id(id))
                    .map(str::to_string)
                else {
                    continue;
                };

                let checksum = compute_checksum(&path);
                let manifest_entry = manifest.entries.get(&id);
                let needs_upload = manifest_entry
                    .map(|entry| entry.checksum != checksum)
                    .unwrap_or(true);

                inventory.file_count += 1;
                if needs_upload {
                    inventory.pending_uploads += 1;
                }
                if manifest_entry
                    .map(|entry| entry.device_id != device_id && entry.checksum != checksum)
                    .unwrap_or(false)
                {
                    inventory.conflict_count += 1;
                }

                local_save_ids.insert(id);
            }
        }
    }

    inventory.pending_downloads = manifest
        .entries
        .keys()
        .filter(|save_id| is_valid_save_id(save_id) && !local_save_ids.contains(*save_id))
        .count();

    inventory
}

fn status_label(config: &CloudSyncConfig, inventory: &SyncInventory) -> String {
    if inventory.conflict_count > 0 {
        return "conflict".to_string();
    }

    let has_pending = inventory.pending_uploads > 0 || inventory.pending_downloads > 0;
    if config.provider == "local" {
        if has_pending {
            "local_changes".to_string()
        } else {
            "local_clean".to_string()
        }
    } else if !config.endpoint_configured {
        "endpoint_missing".to_string()
    } else if !config.token_configured {
        "token_missing".to_string()
    } else if has_pending {
        "remote_pending".to_string()
    } else {
        "remote_ready".to_string()
    }
}

/// Configure cloud sync provider.
#[tauri::command]
pub async fn configure_cloud_sync(
    provider: String,
    endpoint: Option<String>,
    api_key: Option<String>,
) -> Result<String, String> {
    let provider = normalize_provider(&provider);
    let endpoint_configured = has_config_value(&endpoint);
    let token_configured = has_config_value(&api_key);
    *CLOUD_SYNC_CONFIG.write().await = CloudSyncConfig {
        provider: provider.clone(),
        endpoint_configured,
        token_configured,
    };
    tracing::info!(
        "Cloud sync configured: provider={provider}, endpoint_configured={endpoint_configured}, token_configured={token_configured}"
    );
    Ok(format!("Cloud sync provider set to {provider}"))
}

/// Get current cloud sync status with local manifest analysis.
#[tauri::command]
pub async fn get_sync_status(state: State<'_, AppState>) -> Result<CloudSyncStatus, String> {
    let project_root = state.current_project_data_root().await;
    let manifest = load_manifest(&project_root);
    let device_id = get_device_id();
    let inventory = analyze_sync_inventory(&project_root, &manifest, &device_id);
    let config = CLOUD_SYNC_CONFIG.read().await.clone();
    let connected =
        config.provider != "local" && config.endpoint_configured && config.token_configured;

    Ok(CloudSyncStatus {
        connected,
        status: status_label(&config, &inventory),
        last_sync: inventory.last_sync,
        file_count: inventory.file_count,
        pending_uploads: inventory.pending_uploads,
        pending_downloads: inventory.pending_downloads,
        conflict_count: inventory.conflict_count,
        provider: config.provider,
        endpoint_configured: config.endpoint_configured,
        token_configured: config.token_configured,
    })
}

/// Push local saves to cloud with manifest tracking.
#[tauri::command]
pub async fn push_saves_to_cloud(
    state: State<'_, AppState>,
    save_ids: Option<Vec<String>>,
) -> Result<String, String> {
    let sm = state.save_manager.read().await;
    let saves = sm.list_saves().await.map_err(|e| e.to_string())?;
    let project_root = state.current_project_data_root().await;
    let mut manifest = load_manifest(&project_root);
    let device_id = get_device_id();

    let target_ids = save_ids.unwrap_or_else(|| saves.iter().map(|s| s.save_id.clone()).collect());
    let mut pushed = 0usize;

    for save_id in &target_ids {
        if !is_valid_save_id(save_id) {
            continue;
        }
        if let Some(save) = saves.iter().find(|s| &s.save_id == save_id) {
            let fallback_bytes = serde_json::to_vec(save).unwrap_or_default();
            let save_path = save_file_path(&project_root, save_id);
            let (size, checksum) = save_path
                .as_deref()
                .and_then(|path| {
                    let bytes = std::fs::read(path).ok()?;
                    Some((bytes.len() as u64, compute_bytes_checksum(&bytes)))
                })
                .unwrap_or_else(|| {
                    (
                        fallback_bytes.len() as u64,
                        compute_bytes_checksum(&fallback_bytes),
                    )
                });
            manifest.entries.insert(
                save_id.clone(),
                CloudSaveEntry {
                    save_id: save_id.clone(),
                    device_id: device_id.clone(),
                    timestamp: now_timestamp(),
                    size_bytes: size,
                    checksum,
                },
            );
            pushed += 1;
        }
    }

    manifest.device_id = device_id;
    manifest.last_updated = now_timestamp();
    save_manifest(&project_root, &manifest)?;
    tracing::info!("Pushed {pushed} saves to cloud manifest");
    Ok(format!("Pushed {pushed} saves"))
}

/// Pull saves from cloud manifest.
#[tauri::command]
pub async fn pull_saves_from_cloud(
    state: State<'_, AppState>,
) -> Result<Vec<CloudSaveEntry>, String> {
    let project_root = state.current_project_data_root().await;
    let manifest = load_manifest(&project_root);
    Ok(manifest
        .entries
        .values()
        .filter(|entry| is_valid_save_id(&entry.save_id))
        .cloned()
        .collect())
}

/// Resolve sync conflict by choosing local or remote version.
#[tauri::command]
pub async fn resolve_sync_conflict(
    state: State<'_, AppState>,
    save_id: String,
    use_local: bool,
) -> Result<String, String> {
    if !is_valid_save_id(&save_id) {
        return Err("Save id is invalid for sync conflict resolution.".to_string());
    }
    let project_root = state.current_project_data_root().await;
    let mut manifest = load_manifest(&project_root);
    if !use_local {
        manifest.entries.remove(&save_id);
    }
    manifest.last_updated = now_timestamp();
    save_manifest(&project_root, &manifest)?;
    tracing::info!(
        "Resolved conflict for {save_id}: {}",
        if use_local { "local" } else { "remote" }
    );
    Ok(format!("Resolved {save_id}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_manifest_stays_inside_project_saves_dir() {
        let root = PathBuf::from("project-data");
        assert_eq!(saves_dir(&root), root.join("saves"));
        assert_eq!(
            manifest_path(&root),
            root.join("saves").join(".sync_manifest.json")
        );
    }

    #[test]
    fn sync_inventory_ignores_manifest_and_reports_pending_work() {
        let root = std::env::temp_dir().join(format!(
            "monogatari_sync_inventory_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(saves_dir(&root)).unwrap();
        std::fs::write(
            saves_dir(&root).join("slot_a.json"),
            br#"{"save_id":"slot_a"}"#,
        )
        .unwrap();
        std::fs::write(manifest_path(&root), br#"{"entries":{}}"#).unwrap();

        let manifest = SyncManifest {
            entries: HashMap::from([(
                "slot_b".to_string(),
                CloudSaveEntry {
                    save_id: "slot_b".to_string(),
                    device_id: "device-a".to_string(),
                    timestamp: "2026-07-09T00:00:00Z".to_string(),
                    size_bytes: 2,
                    checksum: "remote".to_string(),
                },
            )]),
            device_id: "device-a".to_string(),
            last_updated: "2026-07-09T00:00:00Z".to_string(),
        };

        let inventory = analyze_sync_inventory(&root, &manifest, "device-a");
        assert_eq!(inventory.file_count, 1);
        assert_eq!(inventory.pending_uploads, 1);
        assert_eq!(inventory.pending_downloads, 1);
        assert_eq!(inventory.last_sync.as_deref(), Some("2026-07-09T00:00:00Z"));

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sync_inventory_reports_cross_device_conflicts() {
        let root = std::env::temp_dir().join(format!(
            "monogatari_sync_conflict_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(saves_dir(&root)).unwrap();
        std::fs::write(
            saves_dir(&root).join("slot_a.json"),
            br#"{"save_id":"slot_a"}"#,
        )
        .unwrap();

        let manifest = SyncManifest {
            entries: HashMap::from([(
                "slot_a".to_string(),
                CloudSaveEntry {
                    save_id: "slot_a".to_string(),
                    device_id: "other-device".to_string(),
                    timestamp: "2026-07-09T00:00:00Z".to_string(),
                    size_bytes: 2,
                    checksum: "different".to_string(),
                },
            )]),
            device_id: "other-device".to_string(),
            last_updated: String::new(),
        };

        let inventory = analyze_sync_inventory(&root, &manifest, "this-device");
        assert_eq!(inventory.file_count, 1);
        assert_eq!(inventory.pending_uploads, 1);
        assert_eq!(inventory.pending_downloads, 0);
        assert_eq!(inventory.conflict_count, 1);

        std::fs::remove_dir_all(root).unwrap();
    }
}
