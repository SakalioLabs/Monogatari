use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;

use super::*;

static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(0);

struct TestProject {
    root: PathBuf,
}

impl TestProject {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "monogatari-campaign-preview-{}-{}",
            std::process::id(),
            NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(root.join("campaigns")).unwrap();
        std::fs::create_dir_all(root.join("roleplays")).unwrap();
        std::fs::create_dir_all(root.join("characters")).unwrap();
        std::fs::write(
            root.join("campaigns/story.json"),
            br#"{
                "schema":"monogatari-roleplay-campaign/v1",
                "id":"story",
                "title":"Story",
                "start_entry_id":"chapter",
                "entries":[{
                    "id":"chapter",
                    "roleplay_id":"chapter_roleplay",
                    "routes":[
                        {"ending_id":"good","target":{"kind":"complete"}},
                        {"ending_id":"quiet","target":{"kind":"complete"}}
                    ]
                }]
            }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("roleplays/chapter.json"),
            br#"{
                "schema":"monogatari-scene-roleplay/v1",
                "id":"chapter_roleplay",
                "title":"Chapter",
                "start_node_id":"contact",
                "exhaustion_ending_id":"quiet",
                "max_total_turns":2,
                "score_dimensions":[{
                    "id":"trust",
                    "label":"Trust",
                    "description":"Evidence-backed trust.",
                    "min":0.0,
                    "max":1.0,
                    "initial":0.0
                }],
                "nodes":[{
                    "id":"contact",
                    "scene_id":"station",
                    "character_id":"aoi",
                    "opening_narration":"The signal opens.",
                    "situation":"Aoi asks for a test.",
                    "player_goal":"Offer a test.",
                    "character_goal":"Protect uncertain facts.",
                    "intrusion_response":{
                        "reality_anchors":["The receiver light is blinking."],
                        "interpretations":["That answer belongs elsewhere."],
                        "redirects":["Describe a test in this room."]
                    },
                    "min_turns":1,
                    "max_turns":1,
                    "score_rules":[{
                        "dimension_id":"trust",
                        "guidance":"Reward verification.",
                        "max_delta_per_turn":1.0
                    }],
                    "evidence_rules":[{
                        "id":"plan",
                        "description":"The player proposes a test."
                    }],
                    "transitions":[{
                        "id":"verified",
                        "priority":10,
                        "target":{"kind":"ending","ending_id":"good"},
                        "conditions":[
                            {"kind":"score_at_least","dimension_id":"trust","value":1.0},
                            {"kind":"evidence_observed","evidence_id":"plan"}
                        ]
                    }],
                    "timeout_target":{"kind":"ending","ending_id":"quiet"}
                }]
            }"#,
        )
        .unwrap();
        Self { root }
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn successful_turn() -> SceneRoleplayTurnInput {
    serde_json::from_value(json!({
        "player_message":"Repeat the test.",
        "npc_response":"Then the result can stand on evidence.",
        "evaluation":{
            "score_deltas":[{
                "dimension_id":"trust",
                "delta":1.0,
                "reason":"Repeatable evidence."
            }],
            "evidence":[{
                "evidence_id":"plan",
                "player_quote":"Repeat the test"
            }],
            "npc_emotion":"focused",
            "summary":"A repeatable test was proposed."
        }
    }))
    .unwrap()
}

#[test]
fn campaign_preview_replays_roleplay_and_seals_route_evidence() {
    let project = TestProject::new();
    let preview = execute_project_campaign_preview(
        &project.root,
        "campaigns/story.json",
        vec![CampaignEntryPreviewInput {
            entry_id: "chapter".to_string(),
            turns: vec![successful_turn()],
        }],
    )
    .unwrap();

    assert!(preview.report.completed);
    assert_eq!(preview.report.traversed_routes, ["chapter:good"]);
    assert_eq!(
        preview.report.final_session.completed_entries[0].scores["trust"],
        1.0
    );
    assert_eq!(preview.report.steps[0].roleplay_source_sha256.len(), 64);
    assert_eq!(preview.source_sha256.len(), 64);
}

#[test]
fn campaign_preview_rejects_forged_entry_order() {
    let project = TestProject::new();
    let error = execute_project_campaign_preview(
        &project.root,
        "campaigns/story.json",
        vec![CampaignEntryPreviewInput {
            entry_id: "forged".to_string(),
            turns: vec![successful_turn()],
        }],
    )
    .unwrap_err();

    assert!(error.contains("expected `chapter`"));
}
