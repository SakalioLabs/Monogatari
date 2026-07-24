use llm_game::campaign::{
    RoleplayCampaignEntry, RoleplayCampaignRoute, RoleplayCampaignTarget,
    ROLEPLAY_CAMPAIGN_SCHEMA_V1,
};
use llm_game::scene_roleplay::{
    RoleplayInferenceBudget, RoleplayScoreDimension, RoleplayScoreRule, RoleplayTarget,
    SceneRoleplayNode, SCENE_ROLEPLAY_SCHEMA_V1,
};
use std::sync::atomic::{AtomicU64, Ordering};

use super::*;

static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

fn temp_project(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "monogatari-campaign-validation-{label}-{}-{}",
        std::process::id(),
        NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(root.join("campaigns")).unwrap();
    root
}

fn roleplay() -> LoadedSceneRoleplay {
    LoadedSceneRoleplay {
        definition: SceneRoleplayDefinition {
            schema: SCENE_ROLEPLAY_SCHEMA_V1.to_string(),
            id: "chapter_one".to_string(),
            title: "Chapter One".to_string(),
            start_node_id: "start".to_string(),
            exhaustion_ending_id: "chapter_one_failed".to_string(),
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
                relationship_rule: None,
                evidence_rules: Vec::new(),
                transitions: Vec::new(),
                timeout_target: RoleplayTarget::Ending {
                    ending_id: "chapter_one_ready".to_string(),
                },
            }],
            inference: RoleplayInferenceBudget::default(),
        },
        source_path: "roleplays/chapter_one.json".to_string(),
        absolute_path: PathBuf::from("chapter_one.json"),
    }
}

fn campaign() -> LoadedRoleplayCampaign {
    LoadedRoleplayCampaign {
        definition: RoleplayCampaignDefinition {
            schema: ROLEPLAY_CAMPAIGN_SCHEMA_V1.to_string(),
            id: "volume_one".to_string(),
            title: "Volume One".to_string(),
            start_entry_id: "chapter_one".to_string(),
            entries: vec![RoleplayCampaignEntry {
                id: "chapter_one".to_string(),
                roleplay_id: "chapter_one".to_string(),
                routes: vec![
                    RoleplayCampaignRoute {
                        ending_id: "chapter_one_ready".to_string(),
                        target: RoleplayCampaignTarget::Complete,
                    },
                    RoleplayCampaignRoute {
                        ending_id: "chapter_one_failed".to_string(),
                        target: RoleplayCampaignTarget::Complete,
                    },
                ],
            }],
        },
        source_path: "campaigns/volume_one.json".to_string(),
        absolute_path: PathBuf::from("volume_one.json"),
    }
}

#[test]
fn valid_campaign_routes_every_roleplay_ending() {
    let issues = validate_roleplay_campaign_references(&[campaign()], &[roleplay()]);
    assert!(issues.is_empty());
}

#[test]
fn campaign_reference_evidence_reports_missing_roleplays_and_routes() {
    let mut missing_roleplay = campaign();
    missing_roleplay.definition.entries[0].roleplay_id = "absent".to_string();
    let issues = validate_roleplay_campaign_references(&[missing_roleplay], &[roleplay()]);
    assert_eq!(issues[0].code, "campaign_roleplay_missing");

    let mut incomplete = campaign();
    incomplete.definition.entries[0].routes.pop();
    incomplete.definition.entries[0].routes[0].ending_id = "invented".to_string();
    let issues = validate_roleplay_campaign_references(&[incomplete], &[roleplay()]);
    assert_eq!(
        issues
            .iter()
            .map(|issue| issue.code.as_str())
            .collect::<Vec<_>>(),
        vec![
            "campaign_ending_not_produced",
            "campaign_ending_route_missing",
            "campaign_ending_route_missing"
        ]
    );
}

#[test]
fn project_campaign_loader_is_sorted_and_rejects_duplicate_ids() {
    let root = temp_project("catalog");
    let directory = root.join("campaigns");
    let first = campaign().definition;
    let mut second = first.clone();
    second.id = "another".to_string();
    second.title = "Another".to_string();
    std::fs::write(
        directory.join("z.json"),
        serde_json::to_vec(&first).unwrap(),
    )
    .unwrap();
    std::fs::write(
        directory.join("a.json"),
        serde_json::to_vec(&second).unwrap(),
    )
    .unwrap();

    let loaded = load_project_roleplay_campaigns(&root).unwrap();
    assert_eq!(
        loaded
            .iter()
            .map(|item| item.definition.id.as_str())
            .collect::<Vec<_>>(),
        vec!["another", "volume_one"]
    );

    std::fs::write(
        directory.join("a.json"),
        serde_json::to_vec(&first).unwrap(),
    )
    .unwrap();
    assert!(load_project_roleplay_campaigns(&root)
        .unwrap_err()
        .contains("Duplicate roleplay campaign id"));
    std::fs::remove_dir_all(root).unwrap();
}
