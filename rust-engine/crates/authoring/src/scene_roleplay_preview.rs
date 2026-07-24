//! Deterministic, provider-free scene-roleplay preview execution.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use llm_game::characters::Character;
pub use llm_game::scene_roleplay::SceneRoleplayTurnInput;
use llm_game::scene_roleplay::{
    SceneRoleplayDefinition, SceneRoleplaySession, SceneRoleplayTurnOutcome,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::json_catalog::{
    inspect_project_json_catalog, read_project_json, AuthorableJsonCatalog, JsonCatalogError,
};

pub const SCENE_ROLEPLAY_PREVIEW_SCHEMA_V1: &str = "monogatari-scene-roleplay-preview/v1";
pub const MAX_SCENE_ROLEPLAY_PREVIEW_TURNS: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SceneRoleplayPreviewStep {
    pub turn_index: usize,
    pub outcome: SceneRoleplayTurnOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SceneRoleplayPreviewReport {
    pub schema: String,
    pub roleplay_id: String,
    pub title: String,
    pub executed_turn_count: usize,
    pub completed: bool,
    pub ending_id: Option<String>,
    pub coverage_percent: f32,
    pub intrusion_detected_count: usize,
    pub guarded_response_count: usize,
    pub unguarded_intrusion_count: usize,
    pub visited_node_ids: Vec<String>,
    pub unvisited_node_ids: Vec<String>,
    pub final_session: SceneRoleplaySession,
    pub steps: Vec<SceneRoleplayPreviewStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ProjectSceneRoleplayPreview {
    pub source_path: String,
    pub source_sha256: String,
    pub report: SceneRoleplayPreviewReport,
}

pub fn execute_project_scene_roleplay_preview(
    project_root: &Path,
    requested_path: &str,
    turns: Vec<SceneRoleplayTurnInput>,
) -> Result<ProjectSceneRoleplayPreview, String> {
    let source = read_project_json(project_root, requested_path).map_err(format_catalog_error)?;
    if source.metadata.catalog != AuthorableJsonCatalog::Roleplays {
        return Err(format!(
            "Scene roleplay preview only accepts paths inside `roleplays`; `{requested_path}` targets `{}`.",
            source.metadata.catalog.as_str()
        ));
    }
    let definition =
        serde_json::from_value::<SceneRoleplayDefinition>(source.document).map_err(|error| {
            format!("Scene roleplay `{requested_path}` is not valid schema JSON: {error}")
        })?;
    let initial_relationships = load_initial_player_relationships(project_root, &definition)?;
    let report = execute_scene_roleplay_preview_with_relationships(
        &definition,
        turns,
        initial_relationships,
    )?;
    Ok(ProjectSceneRoleplayPreview {
        source_path: source.metadata.path,
        source_sha256: source.metadata.sha256,
        report,
    })
}

pub fn execute_scene_roleplay_preview(
    definition: &SceneRoleplayDefinition,
    turns: Vec<SceneRoleplayTurnInput>,
) -> Result<SceneRoleplayPreviewReport, String> {
    execute_scene_roleplay_preview_with_relationships(definition, turns, BTreeMap::new())
}

pub fn execute_scene_roleplay_preview_with_relationships(
    definition: &SceneRoleplayDefinition,
    turns: Vec<SceneRoleplayTurnInput>,
    initial_relationships: BTreeMap<String, f32>,
) -> Result<SceneRoleplayPreviewReport, String> {
    if turns.len() > MAX_SCENE_ROLEPLAY_PREVIEW_TURNS {
        return Err(format!(
            "Scene roleplay previews cannot exceed {MAX_SCENE_ROLEPLAY_PREVIEW_TURNS} turns."
        ));
    }
    let mut session =
        SceneRoleplaySession::start_with_relationships(definition, initial_relationships)
            .map_err(|error| error.to_string())?;
    let mut visited_node_ids = vec![definition.start_node_id.clone()];
    let mut steps = Vec::with_capacity(turns.len());

    for (turn_index, turn) in turns.into_iter().enumerate() {
        if session.ending_id.is_some() {
            return Err(format!(
                "Scene roleplay completed before preview turn {turn_index}; remove trailing turns."
            ));
        }
        let outcome = session
            .apply_turn(definition, turn)
            .map_err(|error| format!("Scene roleplay turn {turn_index} failed: {error}"))?;
        if !visited_node_ids.contains(&outcome.current_node_id) {
            visited_node_ids.push(outcome.current_node_id.clone());
        }
        steps.push(SceneRoleplayPreviewStep {
            turn_index,
            outcome,
        });
    }

    let unvisited_node_ids = definition
        .nodes
        .iter()
        .filter(|node| !visited_node_ids.contains(&node.id))
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    let coverage_percent = if definition.nodes.is_empty() {
        0.0
    } else {
        visited_node_ids.len() as f32 / definition.nodes.len() as f32 * 100.0
    };
    let intrusion_detected_count = session
        .transcript
        .iter()
        .filter(|turn| turn.input_safety.intrusion_detected)
        .count();
    let guarded_response_count = session
        .transcript
        .iter()
        .filter(|turn| turn.npc_response_guarded)
        .count();
    let unguarded_intrusion_count = session
        .transcript
        .iter()
        .filter(|turn| turn.input_safety.intrusion_detected && !turn.npc_response_guarded)
        .count();

    Ok(SceneRoleplayPreviewReport {
        schema: SCENE_ROLEPLAY_PREVIEW_SCHEMA_V1.to_string(),
        roleplay_id: definition.id.clone(),
        title: definition.title.clone(),
        executed_turn_count: steps.len(),
        completed: session.ending_id.is_some(),
        ending_id: session.ending_id.clone(),
        coverage_percent,
        intrusion_detected_count,
        guarded_response_count,
        unguarded_intrusion_count,
        visited_node_ids,
        unvisited_node_ids,
        final_session: session,
        steps,
    })
}

pub(crate) fn load_initial_player_relationships(
    project_root: &Path,
    definition: &SceneRoleplayDefinition,
) -> Result<BTreeMap<String, f32>, String> {
    load_initial_player_relationships_for_definitions(project_root, std::iter::once(definition))
}

pub(crate) fn load_initial_player_relationships_for_definitions<'a>(
    project_root: &Path,
    definitions: impl IntoIterator<Item = &'a SceneRoleplayDefinition>,
) -> Result<BTreeMap<String, f32>, String> {
    let relationship_ids = definitions
        .into_iter()
        .flat_map(|definition| definition.nodes.iter())
        .filter(|node| node.relationship_rule.is_some())
        .map(|node| node.character_id.as_str())
        .collect::<BTreeSet<_>>();
    if relationship_ids.is_empty() || !project_root.join("characters").exists() {
        return Ok(BTreeMap::new());
    }

    let catalog =
        inspect_project_json_catalog(project_root, Some(AuthorableJsonCatalog::Characters))
            .map_err(format_catalog_error)?;
    if !catalog.valid {
        return Err("Character catalog is not valid for scene roleplay preview.".to_string());
    }

    let mut relationships = BTreeMap::new();
    for entry in catalog.documents {
        let source = read_project_json(project_root, &entry.path).map_err(format_catalog_error)?;
        let character = serde_json::from_value::<Character>(source.document).map_err(|error| {
            format!(
                "Character `{}` is not valid runtime JSON for scene roleplay preview: {error}",
                entry.path
            )
        })?;
        if relationship_ids.contains(character.id.as_str()) {
            let relationship = character
                .relationships
                .get("player")
                .copied()
                .unwrap_or_default()
                .clamp(-1.0, 1.0);
            relationships.insert(character.id, relationship);
        }
    }
    Ok(relationships)
}

fn format_catalog_error(error: JsonCatalogError) -> String {
    match error.path {
        Some(path) => format!("{} ({path})", error.message),
        None => error.message,
    }
}

#[cfg(test)]
mod tests;
