//! Deterministic, provider-free replay for complete multi-roleplay campaigns.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use llm_game::campaign::{
    RoleplayCampaignAdvance, RoleplayCampaignDefinition, RoleplayCampaignSession,
    RoleplayCampaignStatus,
};
use llm_game::scene_roleplay::SceneRoleplayTurnInput;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::campaign_validation::{
    load_project_roleplay_campaigns, validate_roleplay_campaign_references,
};
use crate::json_catalog::{read_project_json, AuthorableJsonCatalog, JsonCatalogError};
use crate::scene_roleplay_preview::{
    execute_scene_roleplay_preview_with_relationships,
    load_initial_player_relationships_for_definitions, SceneRoleplayPreviewReport,
};
use crate::scene_roleplay_validation::load_project_scene_roleplays;

pub const CAMPAIGN_PREVIEW_SCHEMA_V1: &str = "monogatari-roleplay-campaign-preview/v1";
pub const MAX_CAMPAIGN_PREVIEW_ENTRIES: usize = 256;

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CampaignEntryPreviewInput {
    pub entry_id: String,
    #[serde(default)]
    pub turns: Vec<SceneRoleplayTurnInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CampaignPreviewStep {
    pub entry_id: String,
    pub roleplay_source_path: String,
    pub roleplay_source_sha256: String,
    pub roleplay: SceneRoleplayPreviewReport,
    pub advance: Option<RoleplayCampaignAdvance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CampaignPreviewReport {
    pub schema: String,
    pub campaign_id: String,
    pub title: String,
    pub executed_entry_count: usize,
    pub completed: bool,
    pub current_entry_id: Option<String>,
    pub route_coverage_percent: f32,
    pub traversed_routes: Vec<String>,
    pub untraversed_routes: Vec<String>,
    pub visited_entry_ids: Vec<String>,
    pub unvisited_entry_ids: Vec<String>,
    pub initial_relationships: BTreeMap<String, f32>,
    pub final_session: RoleplayCampaignSession,
    pub steps: Vec<CampaignPreviewStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ProjectCampaignPreview {
    pub source_path: String,
    pub source_sha256: String,
    pub report: CampaignPreviewReport,
}

pub fn execute_project_campaign_preview(
    project_root: &Path,
    requested_path: &str,
    entries: Vec<CampaignEntryPreviewInput>,
) -> Result<ProjectCampaignPreview, String> {
    if entries.len() > MAX_CAMPAIGN_PREVIEW_ENTRIES {
        return Err(format!(
            "Campaign previews cannot exceed {MAX_CAMPAIGN_PREVIEW_ENTRIES} entry runs."
        ));
    }
    let entry_run_count = entries.len();
    let source = read_project_json(project_root, requested_path).map_err(format_catalog_error)?;
    if source.metadata.catalog != AuthorableJsonCatalog::Campaigns {
        return Err(format!(
            "Campaign preview only accepts paths inside `campaigns`; `{requested_path}` targets `{}`.",
            source.metadata.catalog.as_str()
        ));
    }
    let definition = serde_json::from_value::<RoleplayCampaignDefinition>(source.document)
        .map_err(|error| {
            format!("Roleplay campaign `{requested_path}` is not valid schema JSON: {error}")
        })?;
    definition.validate().map_err(|error| error.to_string())?;

    let loaded_roleplays = load_project_scene_roleplays(project_root)?;
    let loaded_campaigns = load_project_roleplay_campaigns(project_root)?;
    let loaded_campaign = loaded_campaigns
        .iter()
        .find(|campaign| campaign.source_path == source.metadata.path)
        .ok_or_else(|| {
            format!(
                "Roleplay campaign `{}` is unavailable in the validated catalog.",
                source.metadata.path
            )
        })?;
    let reference_issues = validate_roleplay_campaign_references(
        std::slice::from_ref(loaded_campaign),
        &loaded_roleplays,
    );
    if !reference_issues.is_empty() {
        return Err(reference_issues
            .into_iter()
            .map(|issue| issue.message)
            .collect::<Vec<_>>()
            .join(" "));
    }
    let roleplays = loaded_roleplays
        .iter()
        .map(|loaded| (loaded.definition.id.as_str(), loaded))
        .collect::<HashMap<_, _>>();
    let campaign_roleplays = definition
        .entries
        .iter()
        .filter_map(|entry| roleplays.get(entry.roleplay_id.as_str()))
        .map(|loaded| &loaded.definition)
        .collect::<Vec<_>>();
    let initial_relationships =
        load_initial_player_relationships_for_definitions(project_root, campaign_roleplays)?;
    let mut session = RoleplayCampaignSession::start_with_relationships(
        &definition,
        initial_relationships.clone(),
    )
    .map_err(|error| error.to_string())?;
    let mut steps = Vec::with_capacity(entries.len());
    let mut visited_entry_ids = Vec::new();
    let mut traversed_routes = Vec::new();

    for (index, input) in entries.into_iter().enumerate() {
        let active_entry_id = session.current_entry_id.clone().ok_or_else(|| {
            format!("Campaign completed before preview entry {index}; remove trailing entries.")
        })?;
        if input.entry_id != active_entry_id {
            return Err(format!(
                "Campaign preview entry {index} expected `{active_entry_id}`, received `{}`.",
                input.entry_id
            ));
        }
        let entry = definition
            .entry(&active_entry_id)
            .ok_or_else(|| format!("Campaign entry `{active_entry_id}` is unavailable."))?;
        let loaded = roleplays
            .get(entry.roleplay_id.as_str())
            .ok_or_else(|| format!("Scene roleplay `{}` is unavailable.", entry.roleplay_id))?;
        let roleplay = execute_scene_roleplay_preview_with_relationships(
            &loaded.definition,
            input.turns,
            session.relationships.clone(),
        )?;
        if !visited_entry_ids.contains(&entry.id) {
            visited_entry_ids.push(entry.id.clone());
        }
        let advance = if roleplay.completed {
            let advance = session
                .complete_current_entry(&definition, &loaded.definition, &roleplay.final_session)
                .map_err(|error| error.to_string())?;
            traversed_routes.push(format!(
                "{}:{}",
                advance.completed.entry_id, advance.completed.ending_id
            ));
            Some(advance)
        } else {
            None
        };
        let incomplete = advance.is_none();
        steps.push(CampaignPreviewStep {
            entry_id: entry.id.clone(),
            roleplay_source_path: loaded.source_path.clone(),
            roleplay_source_sha256: read_project_json(project_root, &loaded.source_path)
                .map_err(format_catalog_error)?
                .metadata
                .sha256,
            roleplay,
            advance,
        });
        if incomplete && index + 1 < entry_run_count {
            return Err(format!(
                "Campaign entry `{active_entry_id}` did not complete; remove trailing entry runs."
            ));
        }
    }

    let visited = visited_entry_ids.iter().collect::<BTreeSet<_>>();
    let unvisited_entry_ids = definition
        .entries
        .iter()
        .filter(|entry| !visited.contains(&entry.id))
        .map(|entry| entry.id.clone())
        .collect();
    let all_routes = definition
        .entries
        .iter()
        .flat_map(|entry| {
            entry
                .routes
                .iter()
                .map(move |route| format!("{}:{}", entry.id, route.ending_id))
        })
        .collect::<Vec<_>>();
    let traversed = traversed_routes.iter().collect::<BTreeSet<_>>();
    let untraversed_routes = all_routes
        .iter()
        .filter(|route| !traversed.contains(route))
        .cloned()
        .collect::<Vec<_>>();
    let route_coverage_percent = if all_routes.is_empty() {
        0.0
    } else {
        traversed_routes.len() as f32 / all_routes.len() as f32 * 100.0
    };

    Ok(ProjectCampaignPreview {
        source_path: source.metadata.path,
        source_sha256: source.metadata.sha256,
        report: CampaignPreviewReport {
            schema: CAMPAIGN_PREVIEW_SCHEMA_V1.to_string(),
            campaign_id: definition.id,
            title: definition.title,
            executed_entry_count: steps.len(),
            completed: session.status == RoleplayCampaignStatus::Completed,
            current_entry_id: session.current_entry_id.clone(),
            route_coverage_percent,
            traversed_routes,
            untraversed_routes,
            visited_entry_ids,
            unvisited_entry_ids,
            initial_relationships,
            final_session: session,
            steps,
        },
    })
}

fn format_catalog_error(error: JsonCatalogError) -> String {
    match error.path {
        Some(path) => format!("{} ({path})", error.message),
        None => error.message,
    }
}

#[cfg(test)]
mod tests;
