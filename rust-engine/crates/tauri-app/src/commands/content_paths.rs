//! Tauri adapter for shared project content path resolution.

use std::path::PathBuf;

use llm_authoring::paths::project_content_dir;

use crate::state::AppState;

pub(crate) async fn resolve_project_content_dir(
    state: &AppState,
    requested_dir: &str,
    canonical_dir: &str,
) -> Result<PathBuf, String> {
    let project_root = state.current_project_data_root().await;
    project_content_dir(&project_root, requested_dir, canonical_dir)
}
