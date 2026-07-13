//! Machine-readable delivery asset readiness built on the shared runtime validator.

use std::path::{Component, Path};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::runtime_validation::{load_core_runtime_project, CoreRuntimeValidationReport};
use crate::story_content_validation::load_scene_documents;

pub const DELIVERY_VALIDATION_SCHEMA_V1: &str = "monogatari-delivery-validation/v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryIssueSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct DeliveryValidationIssue {
    pub severity: DeliveryIssueSeverity,
    pub code: String,
    pub owner_id: Option<String>,
    pub path: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct DeliveryValidationReport {
    pub schema: String,
    pub valid: bool,
    pub core_runtime: CoreRuntimeValidationReport,
    pub declared_renderer_asset_count: usize,
    pub existing_renderer_asset_count: usize,
    pub placeholder_character_count: usize,
    pub declared_audio_asset_count: usize,
    pub existing_audio_asset_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub issues: Vec<DeliveryValidationIssue>,
}

pub async fn validate_project_delivery(
    project_root: &Path,
) -> Result<DeliveryValidationReport, String> {
    let core = load_core_runtime_project(project_root).await?;
    let mut issues = Vec::new();
    let mut declared_renderer_asset_count = 0;
    let mut existing_renderer_asset_count = 0;
    let mut placeholder_character_count = 0;

    for (character_id, character) in core.characters.all_characters() {
        let character = character.read().await;
        let mut assets = Vec::new();
        if let Some(path) = &character.live2d_model_path {
            assets.push(("live2d", path.as_str(), &["json", "model3.json"][..]));
        }
        if let Some(path) = &character.model_3d_path {
            assets.push(("model3d", path.as_str(), &["glb", "gltf"][..]));
        }
        if let Some(path) = &character.portrait_path {
            assets.push(("portrait", path.as_str(), image_extensions()));
        }
        if let Some(path) = &character.sprite_path {
            assets.push(("sprite", path.as_str(), image_extensions()));
        }
        for path in character.sprite_paths.values() {
            assets.push(("expression_sprite", path.as_str(), image_extensions()));
        }
        if assets.is_empty() {
            placeholder_character_count += 1;
            issues.push(warning(
                "character_uses_placeholder",
                Some(character_id.clone()),
                None,
                format!("Character `{character_id}` has no declared renderer asset and will use the placeholder renderer."),
            ));
        }
        for (kind, relative, extensions) in assets {
            declared_renderer_asset_count += 1;
            validate_declared_asset(
                project_root,
                character_id,
                kind,
                relative,
                extensions,
                &mut existing_renderer_asset_count,
                &mut issues,
            );
        }
    }

    let mut declared_audio_asset_count = 0;
    let mut existing_audio_asset_count = 0;
    for loaded in load_scene_documents(project_root)? {
        if let Some(path) = &loaded.scene.bgm_path {
            declared_audio_asset_count += 1;
            validate_declared_asset(
                project_root,
                &loaded.scene.id,
                "scene_bgm",
                path,
                &["mp3", "ogg", "wav", "m4a", "aac", "flac"],
                &mut existing_audio_asset_count,
                &mut issues,
            );
        }
    }

    issues.sort_by(|left, right| {
        left.owner_id
            .cmp(&right.owner_id)
            .then_with(|| left.code.cmp(&right.code))
            .then_with(|| left.path.cmp(&right.path))
    });
    let error_count = issues
        .iter()
        .filter(|issue| issue.severity == DeliveryIssueSeverity::Error)
        .count();
    let warning_count = issues.len() - error_count;
    Ok(DeliveryValidationReport {
        schema: DELIVERY_VALIDATION_SCHEMA_V1.to_string(),
        valid: core.report.valid && error_count == 0,
        core_runtime: core.report,
        declared_renderer_asset_count,
        existing_renderer_asset_count,
        placeholder_character_count,
        declared_audio_asset_count,
        existing_audio_asset_count,
        error_count,
        warning_count,
        issues,
    })
}

fn validate_declared_asset(
    project_root: &Path,
    owner_id: &str,
    kind: &str,
    relative: &str,
    extensions: &[&str],
    existing_count: &mut usize,
    issues: &mut Vec<DeliveryValidationIssue>,
) {
    let resolved = match resolve_asset_path(project_root, relative) {
        Ok(path) => path,
        Err(message) => {
            issues.push(error("asset_path_invalid", owner_id, relative, message));
            return;
        }
    };
    let lower = relative.to_ascii_lowercase();
    if !extensions
        .iter()
        .any(|extension| lower.ends_with(&format!(".{extension}")))
    {
        issues.push(error(
            "asset_extension_unsupported",
            owner_id,
            relative,
            format!("{kind} asset `{relative}` uses an unsupported extension."),
        ));
        return;
    }
    if !resolved.is_file() {
        issues.push(error(
            "asset_missing",
            owner_id,
            relative,
            format!("{kind} asset does not exist: {relative}"),
        ));
        return;
    }
    *existing_count += 1;
}

fn resolve_asset_path(project_root: &Path, relative: &str) -> Result<std::path::PathBuf, String> {
    if relative.is_empty()
        || relative.trim() != relative
        || relative.contains('\\')
        || relative.contains(':')
        || relative.chars().any(char::is_control)
        || relative
            .split('/')
            .any(|segment| segment.is_empty() || matches!(segment, "." | ".."))
    {
        return Err("Asset path must use portable non-empty project-relative segments.".into());
    }
    let path = Path::new(relative);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return Err("Asset path cannot escape the project root.".into());
    }
    Ok(project_root.join(path))
}

fn image_extensions() -> &'static [&'static str] {
    &["png", "jpg", "jpeg", "webp", "bmp", "gif", "svg"]
}

fn error(
    code: impl Into<String>,
    owner_id: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> DeliveryValidationIssue {
    DeliveryValidationIssue {
        severity: DeliveryIssueSeverity::Error,
        code: code.into(),
        owner_id: Some(owner_id.into()),
        path: Some(path.into()),
        message: message.into(),
    }
}

fn warning(
    code: impl Into<String>,
    owner_id: Option<String>,
    path: Option<String>,
    message: impl Into<String>,
) -> DeliveryValidationIssue {
    DeliveryValidationIssue {
        severity: DeliveryIssueSeverity::Warning,
        code: code.into(),
        owner_id,
        path,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests;
