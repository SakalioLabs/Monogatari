use std::sync::atomic::{AtomicU64, Ordering};

use llm_game::scene_roleplay::{
    RoleplayCondition, RoleplayEvidenceObservation, RoleplayEvidenceRule, RoleplayInferenceBudget,
    RoleplayScoreDelta, RoleplayScoreDimension, RoleplayScoreRule, RoleplayTarget,
    RoleplayTransitionRule, RoleplayTurnEvaluation, SceneRoleplayDefinition, SceneRoleplayNode,
    SceneRoleplayTurnInput, SCENE_ROLEPLAY_SCHEMA_V1,
};

use super::*;

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

fn temp_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "monogatari-roleplay-preview-{label}-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ))
}

fn definition() -> SceneRoleplayDefinition {
    SceneRoleplayDefinition {
        schema: SCENE_ROLEPLAY_SCHEMA_V1.to_string(),
        id: "test_roleplay".to_string(),
        title: "Test roleplay".to_string(),
        start_node_id: "contact".to_string(),
        exhaustion_ending_id: "quiet_end".to_string(),
        max_total_turns: 4,
        score_dimensions: vec![RoleplayScoreDimension {
            id: "trust".to_string(),
            label: "Trust".to_string(),
            description: "Evidence-backed trust.".to_string(),
            min: -2.0,
            max: 2.0,
            initial: 0.0,
        }],
        nodes: vec![
            SceneRoleplayNode {
                id: "contact".to_string(),
                scene_id: "station".to_string(),
                character_id: "guide".to_string(),
                supporting_character_ids: Vec::new(),
                opening_narration: "The signal opens.".to_string(),
                situation: "A guide asks for verifiable evidence.".to_string(),
                player_goal: "Offer a test.".to_string(),
                character_goal: "Protect uncertain facts.".to_string(),
                knowledge_refs: Vec::new(),
                min_turns: 1,
                max_turns: 2,
                score_rules: vec![RoleplayScoreRule {
                    dimension_id: "trust".to_string(),
                    guidance: "Reward verifiable plans.".to_string(),
                    max_delta_per_turn: 1.0,
                }],
                evidence_rules: vec![RoleplayEvidenceRule {
                    id: "plan".to_string(),
                    description: "The player proposes a repeatable test.".to_string(),
                }],
                transitions: vec![RoleplayTransitionRule {
                    id: "verified".to_string(),
                    priority: 10,
                    target: RoleplayTarget::Node {
                        node_id: "decision".to_string(),
                    },
                    conditions: vec![
                        RoleplayCondition::ScoreAtLeast {
                            dimension_id: "trust".to_string(),
                            value: 1.0,
                        },
                        RoleplayCondition::EvidenceObserved {
                            evidence_id: "plan".to_string(),
                        },
                    ],
                }],
                timeout_target: RoleplayTarget::Node {
                    node_id: "decision".to_string(),
                },
            },
            SceneRoleplayNode {
                id: "decision".to_string(),
                scene_id: "station".to_string(),
                character_id: "guide".to_string(),
                supporting_character_ids: Vec::new(),
                opening_narration: "The decision waits.".to_string(),
                situation: "The evidence is ready for a decision.".to_string(),
                player_goal: "Choose a bounded release.".to_string(),
                character_goal: "Ask about consequences.".to_string(),
                knowledge_refs: Vec::new(),
                min_turns: 1,
                max_turns: 1,
                score_rules: vec![RoleplayScoreRule {
                    dimension_id: "trust".to_string(),
                    guidance: "Reward accountable release plans.".to_string(),
                    max_delta_per_turn: 1.0,
                }],
                evidence_rules: Vec::new(),
                transitions: vec![RoleplayTransitionRule {
                    id: "release".to_string(),
                    priority: 10,
                    target: RoleplayTarget::Ending {
                        ending_id: "open_end".to_string(),
                    },
                    conditions: vec![RoleplayCondition::ScoreAtLeast {
                        dimension_id: "trust".to_string(),
                        value: 2.0,
                    }],
                }],
                timeout_target: RoleplayTarget::Ending {
                    ending_id: "quiet_end".to_string(),
                },
            },
        ],
        inference: RoleplayInferenceBudget::default(),
    }
}

fn turn(delta: f32, evidence: bool) -> SceneRoleplayTurnInput {
    SceneRoleplayTurnInput {
        player_message: "Let another team repeat the measurement.".to_string(),
        npc_response: "Then the claim can remain separate from the witness.".to_string(),
        evaluation: RoleplayTurnEvaluation {
            score_deltas: vec![RoleplayScoreDelta {
                dimension_id: "trust".to_string(),
                delta,
                reason: "Repeatable evidence.".to_string(),
            }],
            evidence: evidence
                .then(|| RoleplayEvidenceObservation {
                    evidence_id: "plan".to_string(),
                    player_quote: "repeat the measurement".to_string(),
                })
                .into_iter()
                .collect(),
            npc_emotion: Some("focused".to_string()),
            summary: "The player proposed verification.".to_string(),
        },
    }
}

#[test]
fn previews_turns_through_the_shared_state_machine() {
    let report =
        execute_scene_roleplay_preview(&definition(), vec![turn(1.0, true), turn(1.0, false)])
            .unwrap();
    assert!(report.completed);
    assert_eq!(report.ending_id.as_deref(), Some("open_end"));
    assert_eq!(report.visited_node_ids, ["contact", "decision"]);
    assert!(report.unvisited_node_ids.is_empty());
    assert_eq!(report.coverage_percent, 100.0);
    assert_eq!(report.final_session.scores["trust"], 2.0);
    assert_eq!(report.steps[0].outcome.current_node_id, "decision");
}

#[test]
fn project_preview_binds_output_to_exact_source_bytes() {
    let root = temp_root("source");
    std::fs::create_dir_all(root.join("roleplays")).unwrap();
    std::fs::write(
        root.join("roleplays/test.json"),
        serde_json::to_vec_pretty(&definition()).unwrap(),
    )
    .unwrap();

    let preview =
        execute_project_scene_roleplay_preview(&root, "roleplays/test.json", vec![turn(1.0, true)])
            .unwrap();
    assert_eq!(preview.source_path, "roleplays/test.json");
    assert_eq!(preview.source_sha256.len(), 64);
    assert_eq!(preview.report.visited_node_ids, ["contact", "decision"]);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_wrong_catalog_and_turns_after_completion() {
    let root = temp_root("wrong-catalog");
    std::fs::create_dir_all(root.join("dialogue")).unwrap();
    std::fs::write(
        root.join("dialogue/test.json"),
        serde_json::to_vec_pretty(&definition()).unwrap(),
    )
    .unwrap();
    let error = execute_project_scene_roleplay_preview(&root, "dialogue/test.json", Vec::new())
        .unwrap_err();
    assert!(error.contains("only accepts paths inside `roleplays`"));

    let error = execute_scene_roleplay_preview(
        &definition(),
        vec![turn(1.0, true), turn(1.0, false), turn(0.0, false)],
    )
    .unwrap_err();
    assert!(error.contains("completed before preview turn 2"));
    std::fs::remove_dir_all(root).unwrap();
}
