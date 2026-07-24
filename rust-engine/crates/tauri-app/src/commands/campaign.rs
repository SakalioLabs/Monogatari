//! Runtime orchestration for deterministic multi-roleplay campaigns.

use std::collections::{BTreeMap, HashMap, HashSet};

use llm_authoring::campaign_validation::load_project_roleplay_campaigns;
use llm_game::campaign::{
    RoleplayCampaignAdvance, RoleplayCampaignDefinition, RoleplayCampaignSession,
    RoleplayCampaignStatus,
};
use llm_game::scene_roleplay::{SceneRoleplayDefinition, SceneRoleplaySession};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::scene_roleplay::{self, SceneRoleplaySnapshot};
use crate::state::AppState;

pub const CAMPAIGN_RUNTIME_SNAPSHOT_SCHEMA_V1: &str =
    "monogatari-roleplay-campaign-runtime-snapshot/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoleplayCampaignRuntimeSnapshot {
    pub schema: String,
    pub definition: RoleplayCampaignDefinition,
    pub session: RoleplayCampaignSession,
    pub active_roleplay: Option<SceneRoleplaySnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_advance: Option<RoleplayCampaignAdvance>,
}

#[tauri::command]
pub async fn list_roleplay_campaigns(
    state: State<'_, AppState>,
) -> Result<Vec<RoleplayCampaignDefinition>, String> {
    load_campaign_definitions(&state).await
}

#[tauri::command]
pub async fn start_roleplay_campaign(
    state: State<'_, AppState>,
    campaign_id: String,
) -> Result<RoleplayCampaignRuntimeSnapshot, String> {
    start_roleplay_campaign_for_state(&state, &campaign_id).await
}

async fn start_roleplay_campaign_for_state(
    state: &AppState,
    campaign_id: &str,
) -> Result<RoleplayCampaignRuntimeSnapshot, String> {
    let definition = load_campaign_definition(state, campaign_id).await?;
    let roleplays = roleplay_map(state).await?;
    let relationships = load_campaign_relationships(state, &definition, &roleplays).await?;
    let session = RoleplayCampaignSession::start_with_relationships(&definition, relationships)
        .map_err(|error| error.to_string())?;
    let active_roleplay = start_active_roleplay(state, &definition, &session, &roleplays).await?;
    state
        .roleplay_campaign_sessions
        .write()
        .await
        .insert(definition.id.clone(), session.clone());
    Ok(runtime_snapshot(
        definition,
        session,
        Some(active_roleplay),
        None,
    ))
}

#[tauri::command]
pub async fn get_roleplay_campaign_state(
    state: State<'_, AppState>,
    campaign_id: String,
) -> Result<Option<RoleplayCampaignRuntimeSnapshot>, String> {
    let definition = load_campaign_definition(&state, &campaign_id).await?;
    let Some(session) = state
        .roleplay_campaign_sessions
        .read()
        .await
        .get(&definition.id)
        .cloned()
    else {
        return Ok(None);
    };
    session
        .validate(&definition)
        .map_err(|error| error.to_string())?;
    let roleplays = roleplay_map(&state).await?;
    let active_roleplay =
        active_roleplay_snapshot(&state, &definition, &session, &roleplays).await?;
    Ok(Some(runtime_snapshot(
        definition,
        session,
        active_roleplay,
        None,
    )))
}

#[tauri::command]
pub async fn advance_roleplay_campaign(
    state: State<'_, AppState>,
    campaign_id: String,
) -> Result<RoleplayCampaignRuntimeSnapshot, String> {
    advance_roleplay_campaign_for_state(&state, &campaign_id).await
}

async fn advance_roleplay_campaign_for_state(
    state: &AppState,
    campaign_id: &str,
) -> Result<RoleplayCampaignRuntimeSnapshot, String> {
    let definition = load_campaign_definition(state, campaign_id).await?;
    let roleplays = roleplay_map(state).await?;
    let mut session = state
        .roleplay_campaign_sessions
        .read()
        .await
        .get(&definition.id)
        .cloned()
        .ok_or_else(|| {
            format!(
                "Roleplay campaign `{}` has not been started.",
                definition.id
            )
        })?;
    session
        .validate(&definition)
        .map_err(|error| error.to_string())?;
    if session.status == RoleplayCampaignStatus::Completed {
        return Err(format!(
            "Roleplay campaign `{}` is already completed.",
            definition.id
        ));
    }
    let entry_id = session
        .current_entry_id
        .as_deref()
        .ok_or_else(|| "Active campaign has no current entry.".to_string())?;
    let entry = definition
        .entry(entry_id)
        .ok_or_else(|| format!("Campaign entry `{entry_id}` is unavailable."))?;
    let roleplay_definition = roleplays
        .get(&entry.roleplay_id)
        .ok_or_else(|| format!("Scene roleplay `{}` is unavailable.", entry.roleplay_id))?;
    let roleplay_session = state
        .scene_roleplay_sessions
        .read()
        .await
        .get(&entry.roleplay_id)
        .cloned()
        .ok_or_else(|| {
            format!(
                "Scene roleplay `{}` has not been started.",
                entry.roleplay_id
            )
        })?;
    let advance = session
        .complete_current_entry(&definition, roleplay_definition, &roleplay_session)
        .map_err(|error| error.to_string())?;
    let active_roleplay = if session.status == RoleplayCampaignStatus::Active {
        Some(start_active_roleplay(state, &definition, &session, &roleplays).await?)
    } else {
        None
    };
    state
        .roleplay_campaign_sessions
        .write()
        .await
        .insert(definition.id.clone(), session.clone());
    Ok(runtime_snapshot(
        definition,
        session,
        active_roleplay,
        Some(advance),
    ))
}

pub(crate) async fn load_campaign_definitions(
    state: &AppState,
) -> Result<Vec<RoleplayCampaignDefinition>, String> {
    let root = state.current_project_data_root().await;
    load_project_roleplay_campaigns(&root)
        .map(|loaded| loaded.into_iter().map(|loaded| loaded.definition).collect())
}

async fn load_campaign_definition(
    state: &AppState,
    campaign_id: &str,
) -> Result<RoleplayCampaignDefinition, String> {
    let campaign_id = campaign_id.trim();
    if campaign_id.is_empty() {
        return Err("Roleplay campaign id is required.".to_string());
    }
    load_campaign_definitions(state)
        .await?
        .into_iter()
        .find(|definition| definition.id == campaign_id)
        .ok_or_else(|| format!("Roleplay campaign `{campaign_id}` was not found."))
}

async fn roleplay_map(
    state: &AppState,
) -> Result<HashMap<String, SceneRoleplayDefinition>, String> {
    Ok(scene_roleplay::load_definitions(state)
        .await?
        .into_iter()
        .map(|definition| (definition.id.clone(), definition))
        .collect())
}

async fn load_campaign_relationships(
    state: &AppState,
    campaign: &RoleplayCampaignDefinition,
    roleplays: &HashMap<String, SceneRoleplayDefinition>,
) -> Result<BTreeMap<String, f32>, String> {
    let mut character_ids = HashSet::new();
    for entry in &campaign.entries {
        let roleplay = roleplays
            .get(&entry.roleplay_id)
            .ok_or_else(|| format!("Scene roleplay `{}` is unavailable.", entry.roleplay_id))?;
        character_ids.extend(
            roleplay
                .nodes
                .iter()
                .filter(|node| node.relationship_rule.is_some())
                .map(|node| node.character_id.clone()),
        );
    }
    let characters = state.character_manager.read().await;
    let mut relationships = BTreeMap::new();
    for character_id in character_ids {
        let character = characters
            .get_character(&character_id)
            .ok_or_else(|| format!("Character `{character_id}` is not loaded."))?;
        let value = character
            .read()
            .await
            .relationships
            .get("player")
            .copied()
            .unwrap_or_default();
        relationships.insert(character_id, value);
    }
    Ok(relationships)
}

async fn start_active_roleplay(
    state: &AppState,
    campaign: &RoleplayCampaignDefinition,
    session: &RoleplayCampaignSession,
    roleplays: &HashMap<String, SceneRoleplayDefinition>,
) -> Result<SceneRoleplaySnapshot, String> {
    let entry_id = session
        .current_entry_id
        .as_deref()
        .ok_or_else(|| "Active campaign has no current entry.".to_string())?;
    let entry = campaign
        .entry(entry_id)
        .ok_or_else(|| format!("Campaign entry `{entry_id}` is unavailable."))?;
    let definition = roleplays
        .get(&entry.roleplay_id)
        .cloned()
        .ok_or_else(|| format!("Scene roleplay `{}` is unavailable.", entry.roleplay_id))?;
    let roleplay_session =
        SceneRoleplaySession::start_with_relationships(&definition, session.relationships.clone())
            .map_err(|error| error.to_string())?;
    state
        .scene_roleplay_sessions
        .write()
        .await
        .insert(definition.id.clone(), roleplay_session.clone());
    scene_roleplay::snapshot(definition, roleplay_session)
}

async fn active_roleplay_snapshot(
    state: &AppState,
    campaign: &RoleplayCampaignDefinition,
    session: &RoleplayCampaignSession,
    roleplays: &HashMap<String, SceneRoleplayDefinition>,
) -> Result<Option<SceneRoleplaySnapshot>, String> {
    let Some(entry_id) = session.current_entry_id.as_deref() else {
        return Ok(None);
    };
    let entry = campaign
        .entry(entry_id)
        .ok_or_else(|| format!("Campaign entry `{entry_id}` is unavailable."))?;
    let definition = roleplays
        .get(&entry.roleplay_id)
        .cloned()
        .ok_or_else(|| format!("Scene roleplay `{}` is unavailable.", entry.roleplay_id))?;
    let roleplay_session = state
        .scene_roleplay_sessions
        .read()
        .await
        .get(&entry.roleplay_id)
        .cloned();
    roleplay_session
        .map(|roleplay_session| scene_roleplay::snapshot(definition, roleplay_session))
        .transpose()
}

fn runtime_snapshot(
    definition: RoleplayCampaignDefinition,
    session: RoleplayCampaignSession,
    active_roleplay: Option<SceneRoleplaySnapshot>,
    last_advance: Option<RoleplayCampaignAdvance>,
) -> RoleplayCampaignRuntimeSnapshot {
    RoleplayCampaignRuntimeSnapshot {
        schema: CAMPAIGN_RUNTIME_SNAPSHOT_SCHEMA_V1.to_string(),
        definition,
        session,
        active_roleplay,
        last_advance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::engine::load_project_content;
    use llm_game::scene_roleplay::{
        evaluate_roleplay_fallback, SceneRoleplayStatus, SceneRoleplayTurnInput,
    };

    async fn loaded_state() -> AppState {
        let state = AppState::new();
        let root = state.current_project_data_root().await;
        let (characters, dialogues, knowledge, events) = load_project_content(&root).await.unwrap();
        *state.character_manager.write().await = characters;
        *state.dialogue_manager.write().await = dialogues;
        *state.knowledge_base.write().await = knowledge;
        *state.story_event_catalog.write().await = events;
        state
    }

    #[tokio::test]
    async fn start_campaign_creates_the_server_owned_first_roleplay() {
        let state = loaded_state().await;
        let campaign = load_campaign_definitions(&state)
            .await
            .unwrap()
            .into_iter()
            .next()
            .unwrap();

        let snapshot = start_roleplay_campaign_for_state(&state, &campaign.id)
            .await
            .unwrap();

        assert_eq!(
            snapshot.session.current_entry_id.as_deref(),
            Some(campaign.start_entry_id.as_str())
        );
        assert_eq!(
            snapshot.active_roleplay.as_ref().unwrap().session.status,
            SceneRoleplayStatus::Active
        );
        assert!(state
            .roleplay_campaign_sessions
            .read()
            .await
            .contains_key(&campaign.id));
    }

    #[tokio::test]
    async fn campaign_advance_uses_the_verified_server_roleplay_ending() {
        let state = loaded_state().await;
        let campaign = load_campaign_definitions(&state)
            .await
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        let started = start_roleplay_campaign_for_state(&state, &campaign.id)
            .await
            .unwrap();
        assert!(advance_roleplay_campaign_for_state(&state, &campaign.id)
            .await
            .is_err());

        let active = started.active_roleplay.unwrap();
        let definition = active.definition;
        let mut session = active.session;
        while session.status == SceneRoleplayStatus::Active {
            let node = definition.node(&session.current_node_id).unwrap();
            let player_message = "Confirm observable facts and preserve responsibility.";
            session
                .apply_turn(
                    &definition,
                    SceneRoleplayTurnInput {
                        player_message: player_message.to_string(),
                        npc_response: "The current scene remains the only authority.".to_string(),
                        evaluation: evaluate_roleplay_fallback(node, player_message),
                    },
                )
                .unwrap();
        }
        state
            .scene_roleplay_sessions
            .write()
            .await
            .insert(definition.id.clone(), session);

        let advanced = advance_roleplay_campaign_for_state(&state, &campaign.id)
            .await
            .unwrap();
        assert_eq!(advanced.session.status, RoleplayCampaignStatus::Completed);
        assert!(advanced.active_roleplay.is_none());
        assert_eq!(advanced.session.completed_entries.len(), 1);
    }
}
