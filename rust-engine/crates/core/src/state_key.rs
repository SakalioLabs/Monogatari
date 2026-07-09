//! Shared validation for persisted script and dialogue state keys.

use std::collections::HashMap;

use crate::{EngineError, Result};

/// Maximum length for script variables, flags, and dialogue state keys.
pub const SCRIPT_STATE_KEY_MAX_CHARS: usize = 128;

/// Normalize a script/dialogue state key before it enters runtime or save data.
pub fn normalize_script_state_key(key: &str) -> Result<String> {
    let key = key.trim();

    if key.is_empty() {
        return Err(EngineError::script(
            "Script state key cannot be empty.",
            0,
            0,
        ));
    }

    if key.chars().count() > SCRIPT_STATE_KEY_MAX_CHARS {
        return Err(EngineError::script(
            format!("Script state key must be {SCRIPT_STATE_KEY_MAX_CHARS} characters or fewer."),
            0,
            0,
        ));
    }

    if key.chars().any(char::is_control) {
        return Err(EngineError::script(
            "Script state key cannot contain control characters.",
            0,
            0,
        ));
    }

    if !key
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
    {
        return Err(EngineError::script(
            "Script state key can contain only ASCII letters, numbers, dots, underscores, or hyphens.",
            0,
            0,
        ));
    }

    if matches!(key, "." | "..") {
        return Err(EngineError::script(
            "Script state key cannot be a current or parent directory marker.",
            0,
            0,
        ));
    }

    Ok(key.to_string())
}

/// Normalize a map of persisted state values and reject ambiguous duplicate keys.
pub fn normalize_script_state_map<T>(values: HashMap<String, T>) -> Result<HashMap<String, T>> {
    let mut normalized = HashMap::with_capacity(values.len());

    for (key, value) in values {
        let key = normalize_script_state_key(&key)?;
        if normalized.insert(key.clone(), value).is_some() {
            return Err(EngineError::script(
                format!("Duplicate script state key after normalization: {key}."),
                0,
                0,
            ));
        }
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_state_keys_trim_and_allow_portable_chars() {
        assert_eq!(
            normalize_script_state_key(" chapter_1.score-passed ").unwrap(),
            "chapter_1.score-passed"
        );
    }

    #[test]
    fn script_state_keys_reject_control_and_path_like_values() {
        for key in [
            "",
            "   ",
            ".",
            "..",
            "bad key",
            "bad/key",
            "bad\\key",
            "bad:key",
            "剧情",
            "score\u{0007}",
        ] {
            assert!(normalize_script_state_key(key).is_err(), "{key:?}");
        }
    }

    #[test]
    fn script_state_keys_reject_oversized_values() {
        let key = "a".repeat(SCRIPT_STATE_KEY_MAX_CHARS + 1);

        assert!(normalize_script_state_key(&key).is_err());
    }

    #[test]
    fn script_state_maps_reject_duplicate_normalized_keys() {
        let values = HashMap::from([(" score ".to_string(), 1), ("score".to_string(), 2)]);

        assert!(normalize_script_state_map(values).is_err());
    }
}
