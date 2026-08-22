use std::collections::{HashMap, HashSet};

use llm_game::dialogue::{Choice, DialogueNode, DialogueScript};

use super::*;

fn node(text: &str, is_ending: bool) -> DialogueNode {
    DialogueNode {
        id: String::new(),
        speaker_id: None,
        scene_id: None,
        text: text.to_string(),
        next_node_id: None,
        choices: Vec::new(),
        condition: None,
        script: None,
        emotion: None,
        use_llm: false,
        llm_prompt: None,
        llm_system_prompt: None,
        response_generation: None,
        free_talk: None,
        is_ending,
        ending_type: None,
    }
}

fn valid_dialogue() -> DialogueScript {
    DialogueScript {
        id: "intro".to_string(),
        title: "Intro".to_string(),
        description: Some("Opening".to_string()),
        start_node_id: "start".to_string(),
        nodes: HashMap::from([("start".to_string(), node("Hello", true))]),
        variables: HashMap::new(),
    }
}

fn codes(result: &DialogueValidationResult) -> HashSet<&str> {
    result
        .issues
        .iter()
        .map(|issue| issue.code.as_str())
        .collect()
}

#[test]
fn normalization_canonicalizes_editable_fields() {
    let mut dialogue = valid_dialogue();
    dialogue.id = " intro ".to_string();
    dialogue.title = " Intro title ".to_string();
    dialogue.description = Some(" Synopsis ".to_string());
    dialogue.start_node_id = " start ".to_string();
    let start = dialogue.nodes.get_mut("start").unwrap();
    start.id = " start ".to_string();
    start.speaker_id = Some(" aoi ".to_string());
    start.scene_id = Some(" archive_room ".to_string());
    start.text = " Hello world. ".to_string();
    start.emotion = Some(" calm ".to_string());

    let normalized = normalize_dialogue_script(dialogue).unwrap();

    assert_eq!(normalized.id, "intro");
    assert_eq!(normalized.title, "Intro title");
    assert_eq!(normalized.description.as_deref(), Some("Synopsis"));
    assert_eq!(normalized.start_node_id, "start");
    let start = &normalized.nodes["start"];
    assert!(start.id.is_empty());
    assert_eq!(start.speaker_id.as_deref(), Some("aoi"));
    assert_eq!(start.scene_id.as_deref(), Some("archive_room"));
    assert_eq!(start.text, "Hello world.");
    assert_eq!(start.emotion.as_deref(), Some("calm"));
}

#[test]
fn normalization_rejects_relationship_aliases() {
    let mut dialogue = valid_dialogue();
    let mut end = node("Done", true);
    let mut start = node("Choose", false);
    start.choices.push(Choice {
        text: "Go".to_string(),
        next_node_id: "end".to_string(),
        relationship_changes: HashMap::from([("aoi".to_string(), 0.1), (" aoi ".to_string(), 0.2)]),
        condition: None,
    });
    end.id = "end".to_string();
    dialogue.nodes = HashMap::from([("start".to_string(), start), ("end".to_string(), end)]);

    let error = normalize_dialogue_script(dialogue).unwrap_err();

    assert!(error.contains("duplicate relationship target `aoi`"));
}

#[test]
fn validation_reports_character_and_relationship_failures() {
    let mut dialogue = valid_dialogue();
    let mut start = node("Choose", false);
    start.speaker_id = Some("missing_speaker".to_string());
    start.choices.push(Choice {
        text: "Go".to_string(),
        next_node_id: "end".to_string(),
        relationship_changes: HashMap::from([("missing_target".to_string(), 1.5)]),
        condition: None,
    });
    dialogue.nodes = HashMap::from([
        ("start".to_string(), start),
        ("end".to_string(), node("Done", true)),
    ]);

    let result = validate_dialogue_script(&dialogue, &HashSet::new());
    let issue_codes = codes(&result);

    assert!(!result.valid);
    assert!(issue_codes.contains("dialogue_speaker_missing"));
    assert!(issue_codes.contains("dialogue_relationship_target_missing"));
    assert!(issue_codes.contains("dialogue_relationship_delta_invalid"));
}

#[test]
fn validation_reports_authoring_limits_beyond_runtime_topology() {
    let mut dialogue = valid_dialogue();
    let start = dialogue.nodes.get_mut("start").unwrap();
    start.text.clear();
    start.use_llm = true;

    let result = validate_dialogue_script(&dialogue, &HashSet::new());
    let issue_codes = codes(&result);

    assert!(!result.valid);
    assert!(issue_codes.contains("dialogue_text_invalid"));
    assert!(issue_codes.contains("dialogue_llm_prompt_missing"));
    assert!(!issue_codes.contains("dialogue_graph_invalid"));
}

#[test]
fn validation_rejects_non_portable_scene_ids() {
    let mut dialogue = valid_dialogue();
    dialogue.nodes.get_mut("start").unwrap().scene_id = Some("../archive".to_string());

    let result = validate_dialogue_script(&dialogue, &HashSet::new());

    assert!(codes(&result).contains("dialogue_scene_id_invalid"));
}

#[test]
fn validation_accepts_a_bounded_referenced_dialogue() {
    let mut dialogue = valid_dialogue();
    dialogue.nodes.get_mut("start").unwrap().speaker_id = Some("aoi".to_string());

    let result = validate_dialogue_script(&dialogue, &HashSet::from(["aoi".to_string()]));

    assert!(result.valid, "{:?}", result.issues);
    assert_eq!(result.error_count, 0);
    assert!(format_dialogue_validation_errors(&result).is_empty());
}

#[test]
fn validation_accepts_controlled_replies_and_contained_free_talk() {
    let mut dialogue = valid_dialogue();
    let start = dialogue.nodes.get_mut("start").unwrap();
    start.speaker_id = Some("aoi".to_string());
    start.response_generation = Some(llm_game::dialogue::DialogueResponseGeneration {
        character_id: None,
        context: "Aoi acknowledges the lantern and keeps the team on the authored path."
            .to_string(),
        grounding_markers: vec!["lantern".to_string()],
        forbidden_markers: vec!["route".to_string()],
        max_characters: 180,
        max_sentences: 2,
    });
    start.free_talk = Some(llm_game::dialogue::DialogueFreeTalk {
        character_id: "aoi".to_string(),
        context: "This is a quiet checkpoint before the team crosses the bridge.".to_string(),
        title: Some("Talk with Aoi".to_string()),
        opening_text: Some("The bridge is still ahead of us.".to_string()),
        fallback_text: "Aoi nods and points back toward the bridge.".to_string(),
        max_turns: 3,
        max_characters: 180,
    });

    let result = validate_dialogue_script(&dialogue, &HashSet::from(["aoi".to_string()]));

    assert!(result.valid, "{:?}", result.issues);
}

#[test]
fn validation_rejects_unbounded_or_ambiguous_generation_contracts() {
    let mut dialogue = valid_dialogue();
    let start = dialogue.nodes.get_mut("start").unwrap();
    start.use_llm = true;
    start.response_generation = Some(llm_game::dialogue::DialogueResponseGeneration {
        character_id: None,
        context: String::new(),
        grounding_markers: vec!["same".to_string(), "same".to_string()],
        forbidden_markers: Vec::new(),
        max_characters: 10,
        max_sentences: 0,
    });
    start.free_talk = Some(llm_game::dialogue::DialogueFreeTalk {
        character_id: "missing".to_string(),
        context: String::new(),
        title: None,
        opening_text: None,
        fallback_text: String::new(),
        max_turns: 0,
        max_characters: 10,
    });

    let result = validate_dialogue_script(&dialogue, &HashSet::new());
    let issue_codes = codes(&result);

    assert!(issue_codes.contains("dialogue_generation_mode_conflict"));
    assert!(issue_codes.contains("dialogue_generation_character_missing"));
    assert!(issue_codes.contains("dialogue_generation_context_invalid"));
    assert!(issue_codes.contains("dialogue_generation_grounding_markers_invalid"));
    assert!(issue_codes.contains("dialogue_generation_max_characters_invalid"));
    assert!(issue_codes.contains("dialogue_generation_max_sentences_invalid"));
    assert!(issue_codes.contains("dialogue_free_talk_character_missing"));
    assert!(issue_codes.contains("dialogue_free_talk_context_invalid"));
    assert!(issue_codes.contains("dialogue_free_talk_fallback_invalid"));
    assert!(issue_codes.contains("dialogue_free_talk_max_turns_invalid"));
    assert!(issue_codes.contains("dialogue_free_talk_max_characters_invalid"));
}

#[test]
fn validation_evidence_is_deterministic_and_bounded() {
    let mut dialogue = valid_dialogue();
    dialogue.nodes = (0..300)
        .map(|index| {
            let id = format!("node_{index}");
            (id, node("", true))
        })
        .collect();
    dialogue.start_node_id = "node_0".to_string();

    let first = validate_dialogue_script(&dialogue, &HashSet::new());
    let second = validate_dialogue_script(&dialogue, &HashSet::new());

    assert_eq!(first, second);
    assert_eq!(first.error_count, MAX_DIALOGUE_VALIDATION_ISSUES);
    assert!(first
        .issues
        .iter()
        .any(|issue| issue.code == "dialogue_validation_issue_limit_reached"));
}
