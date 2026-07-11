//! Safe, bounded JSON catalog inspection shared by agent transports.

mod inspect;
mod protocol;
mod read;

pub use inspect::inspect_project_json_catalog;
pub use protocol::*;
pub use read::read_project_json;

const MAX_JSON_PATH_BYTES: usize = 512;
const MAX_JSON_PATH_SEGMENTS: usize = 16;
const MAX_JSON_PATH_SEGMENT_BYTES: usize = 128;

/// Validate the portable path policy shared by reads and agent transactions.
pub fn validate_authorable_json_path(
    path: &str,
) -> Result<AuthorableJsonCatalog, JsonCatalogError> {
    if path.len() > MAX_JSON_PATH_BYTES {
        return Err(JsonCatalogError::new(
            JsonCatalogErrorCode::InvalidPath,
            format!("Project JSON paths cannot exceed {MAX_JSON_PATH_BYTES} UTF-8 bytes."),
            Some(path.to_string()),
        ));
    }
    let segments = path.split('/').collect::<Vec<_>>();
    if segments.len() < 2 || segments.len() > MAX_JSON_PATH_SEGMENTS {
        return Err(JsonCatalogError::new(
            JsonCatalogErrorCode::InvalidPath,
            format!("Project JSON paths require 2 to {MAX_JSON_PATH_SEGMENTS} portable segments."),
            Some(path.to_string()),
        ));
    }
    let Some(catalog) = AuthorableJsonCatalog::from_root(segments[0]) else {
        return Err(JsonCatalogError::new(
            JsonCatalogErrorCode::PathNotAllowed,
            "Project JSON paths must target an authorable catalog.",
            Some(path.to_string()),
        ));
    };
    if !path.ends_with(".json") {
        return Err(JsonCatalogError::new(
            JsonCatalogErrorCode::InvalidPath,
            "Authorable documents require a lowercase `.json` extension.",
            Some(path.to_string()),
        ));
    }
    if segments.iter().any(|segment| !is_portable_segment(segment)) {
        return Err(JsonCatalogError::new(
            JsonCatalogErrorCode::InvalidPath,
            "Project JSON paths must use bounded portable ASCII segments.",
            Some(path.to_string()),
        ));
    }
    Ok(catalog)
}

fn is_portable_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment.len() <= MAX_JSON_PATH_SEGMENT_BYTES
        && !segment.starts_with('.')
        && !segment.ends_with('.')
        && !segment.ends_with(' ')
        && segment
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
}

#[cfg(test)]
mod tests;
