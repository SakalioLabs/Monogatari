use super::*;
use crate::scene_roleplay::{
    RoleplayInferenceBudget, RoleplayRelationshipRule, RoleplayScoreDelta, RoleplayScoreDimension,
    RoleplayScoreRule, RoleplayTarget, RoleplayTurnEvaluation, SceneRoleplayNode,
    SceneRoleplayTurnInput,
};

fn definition() -> RoleplayCampaignDefinition {
    RoleplayCampaignDefinition {
        schema: ROLEPLAY_CAMPAIGN_SCHEMA_V1.to_string(),
        id: "volume_one".to_string(),
        title: "Volume One".to_string(),
        start_entry_id: "chapter_one".to_string(),
        entries: vec![
            RoleplayCampaignEntry {
                id: "chapter_one".to_string(),
                roleplay_id: "chapter1_roleplay".to_string(),
                routes: vec![
                    RoleplayCampaignRoute {
                        ending_id: "chapter1_ready".to_string(),
                        target: RoleplayCampaignTarget::Entry {
                            entry_id: "chapter_two".to_string(),
                        },
                    },
                    RoleplayCampaignRoute {
                        ending_id: "chapter1_failed".to_string(),
                        target: RoleplayCampaignTarget::Complete,
                    },
                ],
            },
            RoleplayCampaignEntry {
                id: "chapter_two".to_string(),
                roleplay_id: "chapter2_roleplay".to_string(),
                routes: vec![RoleplayCampaignRoute {
                    ending_id: "chapter2_done".to_string(),
                    target: RoleplayCampaignTarget::Complete,
                }],
            },
        ],
    }
}

fn roleplay(id: &str, ending_id: &str) -> SceneRoleplayDefinition {
    SceneRoleplayDefinition {
        schema: crate::scene_roleplay::SCENE_ROLEPLAY_SCHEMA_V1.to_string(),
        id: id.to_string(),
        title: id.to_string(),
        start_node_id: "start".to_string(),
        exhaustion_ending_id: ending_id.to_string(),
        max_total_turns: 2,
        score_dimensions: vec![RoleplayScoreDimension {
            id: "trust".to_string(),
            label: "Trust".to_string(),
            description: "Trust".to_string(),
            min: -2.0,
            max: 2.0,
            initial: 0.0,
        }],
        nodes: vec![SceneRoleplayNode {
            id: "start".to_string(),
            scene_id: "room".to_string(),
            character_id: "aqua".to_string(),
            supporting_character_ids: Vec::new(),
            opening_narration: "The scene begins.".to_string(),
            situation: "A decision is required.".to_string(),
            player_goal: "Reach an agreement.".to_string(),
            character_goal: "Protect the party.".to_string(),
            knowledge_refs: Vec::new(),
            intrusion_response: None,
            response_guard: None,
            fallback_evaluation: None,
            min_turns: 1,
            max_turns: 1,
            score_rules: vec![RoleplayScoreRule {
                dimension_id: "trust".to_string(),
                guidance: "Reward clear agreements.".to_string(),
                max_delta_per_turn: 1.0,
            }],
            relationship_rule: Some(RoleplayRelationshipRule {
                guidance: "Reward respect.".to_string(),
                max_delta_per_turn: 0.2,
            }),
            evidence_rules: Vec::new(),
            transitions: Vec::new(),
            timeout_target: RoleplayTarget::Ending {
                ending_id: ending_id.to_string(),
            },
        }],
        inference: RoleplayInferenceBudget::default(),
    }
}

fn completed_roleplay(
    definition: &SceneRoleplayDefinition,
    relationship: f32,
) -> SceneRoleplaySession {
    let mut session = SceneRoleplaySession::start(definition).unwrap();
    session
        .apply_turn(
            definition,
            SceneRoleplayTurnInput {
                player_message: "We have a clear agreement.".to_string(),
                npc_response: "Then we can proceed.".to_string(),
                evaluation: RoleplayTurnEvaluation {
                    score_deltas: vec![RoleplayScoreDelta {
                        dimension_id: "trust".to_string(),
                        delta: 1.0,
                        reason: "clear agreement".to_string(),
                    }],
                    evidence: Vec::new(),
                    relationship_delta: relationship,
                    relationship_reason: "respect".to_string(),
                    npc_emotion: None,
                    summary: "Agreement reached.".to_string(),
                },
            },
        )
        .unwrap();
    session
}

#[test]
fn campaign_advances_only_from_the_completed_active_roleplay() {
    let campaign = definition();
    let mut session = RoleplayCampaignSession::start(&campaign).unwrap();
    let first = roleplay("chapter1_roleplay", "chapter1_ready");
    let completed = completed_roleplay(&first, 0.2);

    let advance = session
        .complete_current_entry(&campaign, &first, &completed)
        .unwrap();

    assert_eq!(
        advance.target,
        RoleplayCampaignTarget::Entry {
            entry_id: "chapter_two".to_string()
        }
    );
    assert_eq!(session.current_entry_id.as_deref(), Some("chapter_two"));
    assert_eq!(session.relationships["aqua"], 0.2);
    assert_eq!(session.completed_entries[0].scores["trust"], 1.0);
    session.validate(&campaign).unwrap();
}

#[test]
fn campaign_completes_on_an_explicit_terminal_route() {
    let campaign = definition();
    let mut session = RoleplayCampaignSession::start(&campaign).unwrap();
    let first = roleplay("chapter1_roleplay", "chapter1_failed");
    let completed = completed_roleplay(&first, -0.2);

    session
        .complete_current_entry(&campaign, &first, &completed)
        .unwrap();

    assert_eq!(session.status, RoleplayCampaignStatus::Completed);
    assert!(session.current_entry_id.is_none());
    assert!(matches!(
        session
            .complete_current_entry(&campaign, &first, &completed)
            .unwrap_err(),
        RoleplayCampaignError::SessionCompleted
    ));
}

#[test]
fn campaign_rejects_forged_jumps_and_unfinished_roleplays() {
    let campaign = definition();
    let mut session = RoleplayCampaignSession::start(&campaign).unwrap();
    session.current_entry_id = Some("chapter_two".to_string());
    assert!(session.validate(&campaign).is_err());

    let mut clean = RoleplayCampaignSession::start(&campaign).unwrap();
    let wrong = roleplay("chapter2_roleplay", "chapter2_done");
    let completed = completed_roleplay(&wrong, 0.0);
    assert!(matches!(
        clean
            .complete_current_entry(&campaign, &wrong, &completed)
            .unwrap_err(),
        RoleplayCampaignError::RoleplayMismatch(_)
    ));

    let right = roleplay("chapter1_roleplay", "chapter1_failed");
    let active = SceneRoleplaySession::start(&right).unwrap();
    assert!(matches!(
        clean
            .complete_current_entry(&campaign, &right, &active)
            .unwrap_err(),
        RoleplayCampaignError::RoleplayMismatch(_)
    ));
}

#[test]
fn campaign_definition_rejects_cycles_and_unreachable_entries() {
    let mut cyclic = definition();
    cyclic.entries[1].routes[0].target = RoleplayCampaignTarget::Entry {
        entry_id: "chapter_one".to_string(),
    };
    assert!(cyclic.validate().is_err());

    let mut unreachable = definition();
    unreachable.entries[0].routes[0].target = RoleplayCampaignTarget::Complete;
    assert!(unreachable.validate().is_err());
}

#[test]
fn restored_session_replays_route_history_instead_of_trusting_cursor_fields() {
    let campaign = definition();
    let mut session = RoleplayCampaignSession::start(&campaign).unwrap();
    let first = roleplay("chapter1_roleplay", "chapter1_ready");
    let completed = completed_roleplay(&first, 0.25);
    session
        .complete_current_entry(&campaign, &first, &completed)
        .unwrap();

    let mut forged = session.clone();
    forged.current_entry_id = Some("chapter_one".to_string());
    assert!(forged.validate(&campaign).is_err());

    let mut forged_history = session;
    forged_history.completed_entries[0].ending_id = "chapter1_failed".to_string();
    assert!(forged_history.validate(&campaign).is_err());
}
