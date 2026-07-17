//! Dialogue normalization and authoring validation shared by desktop and Agent writes.

use std::collections::{HashMap, HashSet};

use llm_core::normalize_script_state_map;
use llm_game::dialogue::DialogueScript;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MAX_DIALOGUE_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_DIALOGUE_NODES: usize = 2048;
pub const MAX_DIALOGUE_CHOICES_PER_NODE: usize = 32;
pub const MAX_RELATIONSHIP_CHANGES_PER_CHOICE: usize = 128;
pub const MAX_DIALOGUE_VARIABLES: usize = 512;
pub const MAX_DIALOGUE_TEXT_CHARS: usize = 16_384;
pub const MAX_DIALOGUE_PROMPT_CHARS: usize = 20_000;
pub const MAX_DIALOGUE_VALIDATION_ISSUES: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DialogueValidationIssue {
    pub code: String,
    pub node_id: Option<String>,
    pub choice_index: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DialogueValidationResult {
    pub valid: bool,
    pub error_count: usize,
    pub issues: Vec<DialogueValidationIssue>,
}

/// Normalize an editable Dialogue into the canonical shape persisted by authoring transports.
pub fn normalize_dialogue_script(mut dialogue: DialogueScript) -> Result<DialogueScript, String> {
    dialogue.id = dialogue.id.trim().to_string();
    dialogue.title = dialogue.title.trim().to_string();
    dialogue.description = normalize_optional(dialogue.description);
    dialogue.start_node_id = dialogue.start_node_id.trim().to_string();
    dialogue.variables = normalize_script_state_map(dialogue.variables)
        .map_err(|error| format!("Dialogue variables are invalid: {error}"))?;
    for (node_id, node) in &mut dialogue.nodes {
        if !node.id.is_empty() && node.id.trim() != node_id {
            return Err(format!(
                "Embedded node id `{}` does not match map key `{node_id}`.",
                node.id
            ));
        }
        node.id.clear();
        node.speaker_id = normalize_optional(node.speaker_id.take());
        node.scene_id = normalize_optional(node.scene_id.take());
        node.text = node.text.trim().to_string();
        node.next_node_id = normalize_optional(node.next_node_id.take());
        node.condition = normalize_optional(node.condition.take());
        node.script = normalize_optional(node.script.take());
        node.emotion = normalize_optional(node.emotion.take());
        node.llm_prompt = normalize_optional(node.llm_prompt.take());
        node.llm_system_prompt = normalize_optional(node.llm_system_prompt.take());
        node.ending_type = normalize_optional(node.ending_type.take());
        for choice in &mut node.choices {
            choice.text = choice.text.trim().to_string();
            choice.next_node_id = choice.next_node_id.trim().to_string();
            choice.condition = normalize_optional(choice.condition.take());
            let mut normalized_changes = HashMap::with_capacity(choice.relationship_changes.len());
            for (character_id, delta) in std::mem::take(&mut choice.relationship_changes) {
                let character_id = character_id.trim().to_string();
                if normalized_changes
                    .insert(character_id.clone(), delta)
                    .is_some()
                {
                    return Err(format!(
                        "Choice has duplicate relationship target `{character_id}` after normalization."
                    ));
                }
            }
            choice.relationship_changes = normalized_changes;
        }
    }
    Ok(dialogue)
}

/// Validate graph, bounded authoring fields, and character references without transport state.
pub fn validate_dialogue_script(
    dialogue: &DialogueScript,
    character_ids: &HashSet<String>,
) -> DialogueValidationResult {
    let mut issues = Vec::new();
    if let Err(error) = dialogue.validate_graph() {
        push_issue(
            &mut issues,
            "dialogue_graph_invalid",
            None,
            None,
            error.to_string(),
        );
    }
    validate_text(
        &mut issues,
        &dialogue.title,
        "dialogue_title_invalid",
        "title",
        1,
        256,
        &dialogue.id,
        None,
        None,
    );
    if let Some(description) = dialogue.description.as_deref() {
        validate_text(
            &mut issues,
            description,
            "dialogue_description_invalid",
            "description",
            1,
            2_048,
            &dialogue.id,
            None,
            None,
        );
    }
    if dialogue.nodes.is_empty() || dialogue.nodes.len() > MAX_DIALOGUE_NODES {
        push_issue(
            &mut issues,
            "dialogue_node_count_invalid",
            None,
            None,
            format!(
                "Dialogue `{}` must contain 1 to {MAX_DIALOGUE_NODES} nodes.",
                dialogue.id
            ),
        );
    }
    if dialogue.variables.len() > MAX_DIALOGUE_VARIABLES {
        push_issue(
            &mut issues,
            "dialogue_variable_count_invalid",
            None,
            None,
            format!(
                "Dialogue `{}` has too many variables; the limit is {MAX_DIALOGUE_VARIABLES}.",
                dialogue.id
            ),
        );
    }
    match serde_json::to_vec(&dialogue.variables) {
        Ok(bytes) if bytes.len() > MAX_DIALOGUE_FILE_BYTES as usize / 2 => push_issue(
            &mut issues,
            "dialogue_variable_bytes_invalid",
            None,
            None,
            format!(
                "Dialogue `{}` variables exceed the catalog size budget.",
                dialogue.id
            ),
        ),
        Err(error) => push_issue(
            &mut issues,
            "dialogue_variables_invalid",
            None,
            None,
            format!("Dialogue variables cannot be serialized: {error}"),
        ),
        Ok(_) => {}
    }

    for (node_id, node) in &dialogue.nodes {
        validate_text(
            &mut issues,
            &node.text,
            "dialogue_text_invalid",
            "text",
            1,
            MAX_DIALOGUE_TEXT_CHARS,
            &dialogue.id,
            Some(node_id),
            None,
        );
        if node.choices.len() > MAX_DIALOGUE_CHOICES_PER_NODE {
            push_issue(
                &mut issues,
                "dialogue_choice_count_invalid",
                Some(node_id),
                None,
                format!(
                    "Dialogue `{}` node `{node_id}` has too many choices; the limit is {MAX_DIALOGUE_CHOICES_PER_NODE}.",
                    dialogue.id
                ),
            );
        }
        if let Some(speaker_id) = node.speaker_id.as_deref() {
            if !character_ids.contains(speaker_id) {
                push_issue(
                    &mut issues,
                    "dialogue_speaker_missing",
                    Some(node_id),
                    None,
                    format!(
                        "Dialogue `{}` node `{node_id}` references unknown speaker `{speaker_id}`.",
                        dialogue.id
                    ),
                );
            }
        }
        if let Some(scene_id) = node.scene_id.as_deref() {
            if !portable_id(scene_id) {
                push_issue(
                    &mut issues,
                    "dialogue_scene_id_invalid",
                    Some(node_id),
                    None,
                    format!(
                        "Dialogue `{}` node `{node_id}` has non-portable scene id `{scene_id}`.",
                        dialogue.id
                    ),
                );
            }
        }
        if let Some(emotion) = node.emotion.as_deref() {
            validate_text(
                &mut issues,
                emotion,
                "dialogue_emotion_invalid",
                "emotion",
                1,
                64,
                &dialogue.id,
                Some(node_id),
                None,
            );
        }
        if let Some(ending_type) = node.ending_type.as_deref() {
            validate_text(
                &mut issues,
                ending_type,
                "dialogue_ending_type_invalid",
                "ending type",
                1,
                64,
                &dialogue.id,
                Some(node_id),
                None,
            );
        }
        if node.use_llm && node.llm_prompt.is_none() {
            push_issue(
                &mut issues,
                "dialogue_llm_prompt_missing",
                Some(node_id),
                None,
                format!(
                    "Dialogue `{}` node `{node_id}` enables LLM generation without an LLM prompt.",
                    dialogue.id
                ),
            );
        }
        for (label, prompt) in [
            ("LLM prompt", node.llm_prompt.as_deref()),
            ("LLM system prompt", node.llm_system_prompt.as_deref()),
        ] {
            if let Some(prompt) = prompt {
                validate_text(
                    &mut issues,
                    prompt,
                    "dialogue_llm_prompt_invalid",
                    label,
                    1,
                    MAX_DIALOGUE_PROMPT_CHARS,
                    &dialogue.id,
                    Some(node_id),
                    None,
                );
            }
        }
        for (choice_index, choice) in node.choices.iter().enumerate() {
            let choice_number = choice_index + 1;
            validate_text(
                &mut issues,
                &choice.text,
                "dialogue_choice_text_invalid",
                &format!("choice {choice_number} text"),
                1,
                2_048,
                &dialogue.id,
                Some(node_id),
                Some(choice_number),
            );
            if choice.relationship_changes.len() > MAX_RELATIONSHIP_CHANGES_PER_CHOICE {
                push_issue(
                    &mut issues,
                    "dialogue_relationship_change_count_invalid",
                    Some(node_id),
                    Some(choice_number),
                    format!(
                        "Dialogue `{}` node `{node_id}` choice {choice_number} has too many relationship changes.",
                        dialogue.id
                    ),
                );
            }
            for (character_id, delta) in &choice.relationship_changes {
                if !character_ids.contains(character_id) {
                    push_issue(
                        &mut issues,
                        "dialogue_relationship_target_missing",
                        Some(node_id),
                        Some(choice_number),
                        format!(
                            "Dialogue `{}` node `{node_id}` choice {choice_number} changes unknown character `{character_id}`.",
                            dialogue.id
                        ),
                    );
                }
                if !delta.is_finite() || !(-1.0..=1.0).contains(delta) {
                    push_issue(
                        &mut issues,
                        "dialogue_relationship_delta_invalid",
                        Some(node_id),
                        Some(choice_number),
                        format!(
                            "Dialogue `{}` node `{node_id}` choice {choice_number} relationship delta for `{character_id}` must be between -1 and 1.",
                            dialogue.id
                        ),
                    );
                }
            }
        }
    }

    issues.sort_by(|left, right| {
        left.node_id
            .cmp(&right.node_id)
            .then_with(|| left.choice_index.cmp(&right.choice_index))
            .then_with(|| left.code.cmp(&right.code))
            .then_with(|| left.message.cmp(&right.message))
    });
    DialogueValidationResult {
        valid: issues.is_empty(),
        error_count: issues.len(),
        issues,
    }
}

fn portable_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.trim() == value
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

pub fn ensure_valid_dialogue_script(
    dialogue: &DialogueScript,
    character_ids: &HashSet<String>,
) -> Result<(), String> {
    let result = validate_dialogue_script(dialogue, character_ids);
    if result.valid {
        Ok(())
    } else {
        Err(format_dialogue_validation_errors(&result))
    }
}

pub fn format_dialogue_validation_errors(result: &DialogueValidationResult) -> String {
    result
        .issues
        .iter()
        .map(|issue| issue.message.as_str())
        .collect::<Vec<_>>()
        .join("; ")
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[allow(clippy::too_many_arguments)]
fn validate_text(
    issues: &mut Vec<DialogueValidationIssue>,
    value: &str,
    code: &str,
    label: &str,
    min: usize,
    max: usize,
    dialogue_id: &str,
    node_id: Option<&str>,
    choice_index: Option<usize>,
) {
    let count = value.chars().count();
    if count < min
        || count > max
        || value
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        let location = node_id
            .map(|node_id| format!(" node `{node_id}`"))
            .unwrap_or_default();
        push_issue(
            issues,
            code,
            node_id,
            choice_index,
            format!(
                "Dialogue `{dialogue_id}`{location} {label} must contain {min} to {max} supported characters."
            ),
        );
    }
}

fn push_issue(
    issues: &mut Vec<DialogueValidationIssue>,
    code: &str,
    node_id: Option<&str>,
    choice_index: Option<usize>,
    message: String,
) {
    if issues.len() >= MAX_DIALOGUE_VALIDATION_ISSUES {
        return;
    }
    if issues.len() == MAX_DIALOGUE_VALIDATION_ISSUES - 1 {
        issues.push(DialogueValidationIssue {
            code: "dialogue_validation_issue_limit_reached".to_string(),
            node_id: None,
            choice_index: None,
            message: format!(
                "Dialogue validation stopped after {MAX_DIALOGUE_VALIDATION_ISSUES} issues."
            ),
        });
        return;
    }
    issues.push(DialogueValidationIssue {
        code: code.to_string(),
        node_id: node_id.map(str::to_string),
        choice_index,
        message,
    });
}

#[cfg(test)]
mod tests;
