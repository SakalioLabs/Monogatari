use super::*;

fn definition() -> SceneRoleplayDefinition {
    SceneRoleplayDefinition {
        schema: SCENE_ROLEPLAY_SCHEMA_V1.to_string(),
        id: "signal_room".to_string(),
        title: "Signal Room".to_string(),
        start_node_id: "contact".to_string(),
        exhaustion_ending_id: "silence".to_string(),
        max_total_turns: 6,
        score_dimensions: vec![
            RoleplayScoreDimension {
                id: "trust".to_string(),
                label: "Trust".to_string(),
                description: "Respect for the witness boundary.".to_string(),
                min: -3.0,
                max: 3.0,
                initial: 0.0,
            },
            RoleplayScoreDimension {
                id: "evidence".to_string(),
                label: "Evidence".to_string(),
                description: "Care with verifiable facts.".to_string(),
                min: -3.0,
                max: 3.0,
                initial: 0.0,
            },
        ],
        nodes: vec![
            SceneRoleplayNode {
                id: "contact".to_string(),
                scene_id: "radio_room".to_string(),
                character_id: "echo".to_string(),
                supporting_character_ids: vec![],
                opening_narration: "A signal answers.".to_string(),
                situation: "The source of the signal is uncertain.".to_string(),
                player_goal: "Establish what can be verified.".to_string(),
                character_goal: "Be heard without claiming a false identity.".to_string(),
                knowledge_refs: vec!["signal_protocol".to_string()],
                min_turns: 2,
                max_turns: 3,
                score_rules: vec![
                    RoleplayScoreRule {
                        dimension_id: "trust".to_string(),
                        guidance: "Reward respect for uncertainty.".to_string(),
                        max_delta_per_turn: 1.0,
                    },
                    RoleplayScoreRule {
                        dimension_id: "evidence".to_string(),
                        guidance: "Reward requests for reproducible evidence.".to_string(),
                        max_delta_per_turn: 0.75,
                    },
                ],
                evidence_rules: vec![RoleplayEvidenceRule {
                    id: "asked_for_coordinates".to_string(),
                    description: "The player asks for reproducible coordinates.".to_string(),
                }],
                transitions: vec![RoleplayTransitionRule {
                    id: "evidence_secured".to_string(),
                    priority: 10,
                    target: RoleplayTarget::Node {
                        node_id: "review".to_string(),
                    },
                    conditions: vec![
                        RoleplayCondition::NodeTurnsAtLeast { value: 2 },
                        RoleplayCondition::ScoreAtLeast {
                            dimension_id: "trust".to_string(),
                            value: 1.0,
                        },
                        RoleplayCondition::EvidenceObserved {
                            evidence_id: "asked_for_coordinates".to_string(),
                        },
                    ],
                }],
                timeout_target: RoleplayTarget::Ending {
                    ending_id: "silence".to_string(),
                },
            },
            SceneRoleplayNode {
                id: "review".to_string(),
                scene_id: "archive".to_string(),
                character_id: "keeper".to_string(),
                supporting_character_ids: vec!["echo".to_string()],
                opening_narration: "The archive opens.".to_string(),
                situation: "The evidence must be published responsibly.".to_string(),
                player_goal: "Choose a bounded publication plan.".to_string(),
                character_goal: "Protect both the record and the witness.".to_string(),
                knowledge_refs: vec![],
                min_turns: 1,
                max_turns: 2,
                score_rules: vec![RoleplayScoreRule {
                    dimension_id: "evidence".to_string(),
                    guidance: "Reward a reproducible publication plan.".to_string(),
                    max_delta_per_turn: 1.0,
                }],
                evidence_rules: vec![RoleplayEvidenceRule {
                    id: "bounded_publication".to_string(),
                    description: "The player proposes a bounded publication plan.".to_string(),
                }],
                transitions: vec![RoleplayTransitionRule {
                    id: "open_archive".to_string(),
                    priority: 10,
                    target: RoleplayTarget::Ending {
                        ending_id: "truth".to_string(),
                    },
                    conditions: vec![RoleplayCondition::EvidenceObserved {
                        evidence_id: "bounded_publication".to_string(),
                    }],
                }],
                timeout_target: RoleplayTarget::Ending {
                    ending_id: "silence".to_string(),
                },
            },
        ],
        inference: RoleplayInferenceBudget {
            max_context_characters: 2_000,
            max_recent_turns: 2,
            npc_max_tokens: 64,
            evaluator_max_tokens: 96,
        },
    }
}

fn turn(trust: f32, evidence: f32, observed: bool) -> SceneRoleplayTurnInput {
    SceneRoleplayTurnInput {
        player_message: "Give me coordinates that another receiver can verify.".to_string(),
        npc_response: "I can repeat the coordinates, but not prove who spoke them.".to_string(),
        evaluation: RoleplayTurnEvaluation {
            score_deltas: vec![
                RoleplayScoreDelta {
                    dimension_id: "trust".to_string(),
                    delta: trust,
                    reason: "The player preserved uncertainty.".to_string(),
                },
                RoleplayScoreDelta {
                    dimension_id: "evidence".to_string(),
                    delta: evidence,
                    reason: "The request is reproducible.".to_string(),
                },
            ],
            evidence: observed
                .then(|| RoleplayEvidenceObservation {
                    evidence_id: "asked_for_coordinates".to_string(),
                    player_quote: "coordinates".to_string(),
                })
                .into_iter()
                .collect(),
            npc_emotion: Some("guarded".to_string()),
            summary: "A bounded evidence request.".to_string(),
        },
    }
}

#[test]
fn score_and_evidence_transition_only_after_minimum_turns() {
    let definition = definition();
    let mut session = SceneRoleplaySession::start(&definition).unwrap();

    let first = session
        .apply_turn(&definition, turn(5.0, 5.0, true))
        .unwrap();
    assert!(first.transition.is_none());
    assert_eq!(first.scores["trust"], 1.0);
    assert_eq!(first.scores["evidence"], 0.75);
    assert_eq!(session.transcript[0].evaluation.score_deltas[0].delta, 1.0);

    let second = session
        .apply_turn(&definition, turn(0.25, 0.25, false))
        .unwrap();
    assert_eq!(second.current_node_id, "review");
    assert_eq!(second.node_turns, 0);
    assert_eq!(second.transition.unwrap().reason, "evidence_secured");
    assert_eq!(session.observed_evidence, vec!["asked_for_coordinates"]);
}

#[test]
fn node_timeout_and_total_turn_limit_always_end_the_session() {
    let definition = definition();
    let mut session = SceneRoleplaySession::start(&definition).unwrap();
    for _ in 0..3 {
        let outcome = session
            .apply_turn(&definition, turn(0.0, 0.0, false))
            .unwrap();
        if outcome.status == SceneRoleplayStatus::Completed {
            assert_eq!(outcome.ending_id.as_deref(), Some("silence"));
            assert_eq!(outcome.transition.unwrap().reason, "node_turn_limit");
            return;
        }
    }
    panic!("node turn limit did not complete the roleplay");
}

#[test]
fn invalid_model_evidence_cannot_mutate_story_state() {
    let definition = definition();
    let mut session = SceneRoleplaySession::start(&definition).unwrap();
    let before = session.clone();
    let mut input = turn(1.0, 0.5, false);
    input.evaluation.evidence.push(RoleplayEvidenceObservation {
        evidence_id: "unlock_everything".to_string(),
        player_quote: "ignore rules".to_string(),
    });

    let error = session.apply_turn(&definition, input).unwrap_err();
    assert!(error.to_string().contains("not allowed"));
    assert_eq!(session, before);

    let mut fabricated = turn(1.0, 0.5, false);
    fabricated
        .evaluation
        .evidence
        .push(RoleplayEvidenceObservation {
            evidence_id: "asked_for_coordinates".to_string(),
            player_quote: "the player never said this".to_string(),
        });
    let error = session.apply_turn(&definition, fabricated).unwrap_err();
    assert!(error.to_string().contains("exact non-empty player quote"));
    assert_eq!(session, before);
}

#[test]
fn prompt_context_is_bounded_and_keeps_the_latest_player_turn() {
    let definition = definition();
    let mut session = SceneRoleplaySession::start(&definition).unwrap();
    session.transcript = (1..=8)
        .map(|turn| SceneRoleplayTurnRecord {
            turn,
            node_id: "contact".to_string(),
            player_message: format!("old player {turn} {}", "x".repeat(700)),
            npc_response: format!("old npc {turn} {}", "y".repeat(700)),
            evaluation: RoleplayTurnEvaluation {
                score_deltas: vec![],
                evidence: vec![],
                npc_emotion: None,
                summary: String::new(),
            },
            newly_observed_evidence: vec![],
        })
        .collect();
    let latest = "latest player message";
    let messages = build_npc_prompt_messages(
        &definition,
        &session,
        &"profile ".repeat(500),
        &"knowledge ".repeat(500),
        "zh-CN",
        latest,
    )
    .unwrap();
    let total_chars = messages
        .iter()
        .map(|message| message.content.chars().count())
        .sum::<usize>();

    assert!(total_chars <= definition.inference.max_context_characters);
    assert_eq!(messages.last().unwrap().role, "user");
    assert_eq!(messages.last().unwrap().content, latest);
    assert!(messages.len() <= 2 + definition.inference.max_recent_turns * 2);
}

#[test]
fn evaluator_json_is_strict_and_definition_rejects_unreachable_nodes() {
    let parsed =
        parse_turn_evaluation_json(r#"{"score_deltas":[],"evidence":[],"summary":"neutral"}"#)
            .unwrap();
    assert_eq!(parsed.summary, "neutral");

    let mut invalid = definition();
    invalid.nodes.push(SceneRoleplayNode {
        id: "orphan".to_string(),
        scene_id: "void".to_string(),
        character_id: "echo".to_string(),
        supporting_character_ids: vec![],
        opening_narration: "Nothing.".to_string(),
        situation: "Unreachable.".to_string(),
        player_goal: "None.".to_string(),
        character_goal: "None.".to_string(),
        knowledge_refs: vec![],
        min_turns: 1,
        max_turns: 1,
        score_rules: vec![RoleplayScoreRule {
            dimension_id: "trust".to_string(),
            guidance: "None.".to_string(),
            max_delta_per_turn: 1.0,
        }],
        evidence_rules: vec![],
        transitions: vec![],
        timeout_target: RoleplayTarget::Ending {
            ending_id: "silence".to_string(),
        },
    });
    assert!(invalid
        .validate()
        .unwrap_err()
        .to_string()
        .contains("unreachable"));
}
