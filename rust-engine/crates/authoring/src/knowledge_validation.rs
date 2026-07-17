//! Knowledge normalization and authoring validation shared by desktop and Agent writes.

use std::collections::HashSet;

use llm_game::knowledge::{KnowledgeCategory, KnowledgeEntry};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MAX_KNOWLEDGE_ENTRIES: usize = 8_192;
pub const MAX_KNOWLEDGE_TAGS: usize = 64;
pub const MAX_KNOWLEDGE_RELATED_ENTRIES: usize = 256;
pub const MAX_KNOWLEDGE_METADATA_BYTES: usize = 256 * 1024;
pub const MAX_KNOWLEDGE_VALIDATION_ISSUES: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeValidationIssue {
    pub code: String,
    pub entry_id: Option<String>,
    pub field: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeValidationResult {
    pub valid: bool,
    pub error_count: usize,
    pub issues: Vec<KnowledgeValidationIssue>,
}

/// Normalize one editable Knowledge entry into the shape persisted by authoring transports.
pub fn normalize_knowledge_entry(mut entry: KnowledgeEntry) -> KnowledgeEntry {
    entry.id = entry.id.trim().to_string();
    entry.category = KnowledgeCategory::from_label(entry.category.as_str());
    entry.title = entry.title.trim().to_string();
    entry.content = entry.content.trim().to_string();

    let mut seen_tags = HashSet::new();
    entry.tags = entry
        .tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty() && seen_tags.insert(tag.to_ascii_lowercase()))
        .collect();

    let mut seen_relations = HashSet::new();
    entry.related_entries = entry
        .related_entries
        .into_iter()
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty() && seen_relations.insert(id.clone()))
        .collect();
    entry
}

/// Validate one entry against the IDs available in its project catalog.
pub fn validate_knowledge_entry(
    entry: &KnowledgeEntry,
    known_ids: &HashSet<String>,
) -> KnowledgeValidationResult {
    let normalized = normalize_knowledge_entry(entry.clone());
    let mut issues = Vec::new();
    validate_canonical_entry(entry, &normalized, &mut issues);
    validate_entry_fields(&normalized, &mut issues);
    validate_entry_relations(&normalized, known_ids, &mut issues);
    finish_validation(issues)
}

/// Validate canonical fields, duplicate IDs, and the complete related-entry closure.
pub fn validate_knowledge_catalog(entries: &[KnowledgeEntry]) -> KnowledgeValidationResult {
    let mut ordered = entries
        .iter()
        .map(|entry| (entry, normalize_knowledge_entry(entry.clone())))
        .collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        left.1
            .id
            .cmp(&right.1.id)
            .then_with(|| left.1.title.cmp(&right.1.title))
            .then_with(|| left.1.content.cmp(&right.1.content))
            .then_with(|| left.1.category.as_str().cmp(right.1.category.as_str()))
    });

    let mut issues = Vec::new();
    if ordered.len() > MAX_KNOWLEDGE_ENTRIES {
        push_issue(
            &mut issues,
            "knowledge_entry_count_invalid",
            None,
            None,
            format!("Knowledge catalogs can contain at most {MAX_KNOWLEDGE_ENTRIES} entries."),
        );
    }

    let known_ids = ordered
        .iter()
        .map(|(_, normalized)| normalized.id.clone())
        .collect::<HashSet<_>>();
    let mut unique_ids = HashSet::new();
    for (source, normalized) in &ordered {
        validate_canonical_entry(source, normalized, &mut issues);
        validate_entry_fields(normalized, &mut issues);
        if !unique_ids.insert(normalized.id.clone()) {
            push_issue(
                &mut issues,
                "duplicate_knowledge_id",
                Some(&normalized.id),
                Some("id"),
                format!("Duplicate knowledge entry id `{}`.", normalized.id),
            );
        }
        validate_entry_relations(normalized, &known_ids, &mut issues);
    }
    finish_validation(issues)
}

pub fn ensure_valid_knowledge_entry(
    entry: &KnowledgeEntry,
    known_ids: &HashSet<String>,
) -> Result<(), String> {
    let result = validate_knowledge_entry(entry, known_ids);
    if result.valid {
        Ok(())
    } else {
        Err(format_knowledge_validation_errors(&result))
    }
}

pub fn ensure_valid_knowledge_catalog(entries: &[KnowledgeEntry]) -> Result<(), String> {
    let result = validate_knowledge_catalog(entries);
    if result.valid {
        Ok(())
    } else {
        Err(format_knowledge_validation_errors(&result))
    }
}

pub fn ensure_valid_knowledge_id(id: &str) -> Result<(), String> {
    if is_portable_knowledge_id(id) {
        Ok(())
    } else {
        Err(
            "Knowledge entry id can contain only 1 to 128 lowercase ASCII letters, numbers, underscores, or hyphens."
                .to_string(),
        )
    }
}

pub fn format_knowledge_validation_errors(result: &KnowledgeValidationResult) -> String {
    result
        .issues
        .iter()
        .map(|issue| issue.message.as_str())
        .collect::<Vec<_>>()
        .join("; ")
}

fn validate_canonical_entry(
    source: &KnowledgeEntry,
    normalized: &KnowledgeEntry,
    issues: &mut Vec<KnowledgeValidationIssue>,
) {
    if source != normalized {
        push_issue(
            issues,
            "knowledge_not_canonical",
            Some(&normalized.id),
            None,
            format!(
                "Knowledge entry `{}` contains non-canonical authoring fields; normalize it before acceptance.",
                normalized.id
            ),
        );
    }
}

fn validate_entry_fields(entry: &KnowledgeEntry, issues: &mut Vec<KnowledgeValidationIssue>) {
    if !is_portable_knowledge_id(&entry.id) {
        push_issue(
            issues,
            "knowledge_id_invalid",
            Some(&entry.id),
            Some("id"),
            format!(
                "Knowledge entry id `{}` must contain 1 to 128 lowercase ASCII letters, numbers, underscores, or hyphens.",
                entry.id
            ),
        );
    }
    validate_text(
        issues,
        entry,
        &entry.title,
        "knowledge_title_invalid",
        "title",
        1,
        256,
        false,
    );
    validate_text(
        issues,
        entry,
        &entry.content,
        "knowledge_content_invalid",
        "content",
        1,
        16_384,
        true,
    );

    let category = entry.category.as_str();
    if category.is_empty()
        || category.len() > 64
        || !category.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
    {
        push_issue(
            issues,
            "knowledge_category_invalid",
            Some(&entry.id),
            Some("category"),
            format!(
                "Knowledge entry `{}` category must contain 1 to 64 lowercase ASCII letters, numbers, underscores, or hyphens.",
                entry.id
            ),
        );
    }
    if !entry.importance.is_finite() || !(0.0..=1.0).contains(&entry.importance) {
        push_issue(
            issues,
            "knowledge_importance_invalid",
            Some(&entry.id),
            Some("importance"),
            format!(
                "Knowledge entry `{}` importance must be between 0 and 1.",
                entry.id
            ),
        );
    }
    if entry.tags.len() > MAX_KNOWLEDGE_TAGS {
        push_issue(
            issues,
            "knowledge_tag_count_invalid",
            Some(&entry.id),
            Some("tags"),
            format!(
                "Knowledge entry `{}` can contain at most {MAX_KNOWLEDGE_TAGS} tags.",
                entry.id
            ),
        );
    }
    for tag in &entry.tags {
        validate_text(
            issues,
            entry,
            tag,
            "knowledge_tag_invalid",
            "tag",
            1,
            64,
            false,
        );
    }
    if entry.related_entries.len() > MAX_KNOWLEDGE_RELATED_ENTRIES {
        push_issue(
            issues,
            "knowledge_relation_count_invalid",
            Some(&entry.id),
            Some("related_entries"),
            format!(
                "Knowledge entry `{}` can relate to at most {MAX_KNOWLEDGE_RELATED_ENTRIES} entries.",
                entry.id
            ),
        );
    }
    for related_id in &entry.related_entries {
        if !is_portable_knowledge_id(related_id) {
            push_issue(
                issues,
                "knowledge_relation_id_invalid",
                Some(&entry.id),
                Some("related_entries"),
                format!(
                    "Knowledge entry `{}` has invalid related entry id `{related_id}`.",
                    entry.id
                ),
            );
        }
    }
    match serde_json::to_vec(&entry.metadata) {
        Ok(bytes) if bytes.len() > MAX_KNOWLEDGE_METADATA_BYTES => push_issue(
            issues,
            "knowledge_metadata_bytes_invalid",
            Some(&entry.id),
            Some("metadata"),
            format!(
                "Knowledge entry `{}` metadata exceeds {MAX_KNOWLEDGE_METADATA_BYTES} bytes.",
                entry.id
            ),
        ),
        Err(error) => push_issue(
            issues,
            "knowledge_metadata_invalid",
            Some(&entry.id),
            Some("metadata"),
            format!(
                "Knowledge entry `{}` metadata cannot be serialized: {error}",
                entry.id
            ),
        ),
        Ok(_) => {}
    }
}

fn validate_entry_relations(
    entry: &KnowledgeEntry,
    known_ids: &HashSet<String>,
    issues: &mut Vec<KnowledgeValidationIssue>,
) {
    let mut related_ids = entry.related_entries.iter().collect::<Vec<_>>();
    related_ids.sort();
    for related_id in related_ids {
        if related_id == &entry.id {
            push_issue(
                issues,
                "knowledge_relation_self",
                Some(&entry.id),
                Some("related_entries"),
                format!("Knowledge entry `{}` cannot reference itself.", entry.id),
            );
        } else if !known_ids.contains(related_id) {
            push_issue(
                issues,
                "knowledge_relation_target_missing",
                Some(&entry.id),
                Some("related_entries"),
                format!(
                    "Knowledge entry `{}` relates to unknown entry `{related_id}`.",
                    entry.id
                ),
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_text(
    issues: &mut Vec<KnowledgeValidationIssue>,
    entry: &KnowledgeEntry,
    value: &str,
    code: &str,
    field: &str,
    min: usize,
    max: usize,
    allow_multiline: bool,
) {
    let length = value.chars().count();
    let has_disallowed_control = value.chars().any(|character| {
        character.is_control() && !(allow_multiline && matches!(character, '\n' | '\r' | '\t'))
    });
    if length < min || length > max || has_disallowed_control {
        push_issue(
            issues,
            code,
            Some(&entry.id),
            Some(field),
            format!(
                "Knowledge entry `{}` {field} must contain {min} to {max} supported characters.",
                entry.id
            ),
        );
    }
}

fn is_portable_knowledge_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn finish_validation(mut issues: Vec<KnowledgeValidationIssue>) -> KnowledgeValidationResult {
    issues.sort_by(|left, right| {
        left.entry_id
            .cmp(&right.entry_id)
            .then_with(|| left.field.cmp(&right.field))
            .then_with(|| left.code.cmp(&right.code))
            .then_with(|| left.message.cmp(&right.message))
    });
    KnowledgeValidationResult {
        valid: issues.is_empty(),
        error_count: issues.len(),
        issues,
    }
}

fn push_issue(
    issues: &mut Vec<KnowledgeValidationIssue>,
    code: &str,
    entry_id: Option<&str>,
    field: Option<&str>,
    message: String,
) {
    if issues.len() >= MAX_KNOWLEDGE_VALIDATION_ISSUES {
        return;
    }
    if issues.len() == MAX_KNOWLEDGE_VALIDATION_ISSUES - 1 {
        issues.push(KnowledgeValidationIssue {
            code: "knowledge_validation_issue_limit_reached".to_string(),
            entry_id: None,
            field: None,
            message: format!(
                "Knowledge validation stopped after {MAX_KNOWLEDGE_VALIDATION_ISSUES} issues."
            ),
        });
        return;
    }
    issues.push(KnowledgeValidationIssue {
        code: code.to_string(),
        entry_id: entry_id.map(str::to_string),
        field: field.map(str::to_string),
        message,
    });
}

#[cfg(test)]
mod tests;
