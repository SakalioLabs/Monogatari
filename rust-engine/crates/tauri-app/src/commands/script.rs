//! Script engine commands.

use tauri::State;

use crate::state::AppState;
use llm_scripting::{validate_condition_source, SCRIPT_MAX_TEXT_CHARS};

const MAX_DSL_SCRIPT_TEXT_CHARS: usize = 100_000;

fn validate_script_text(label: &str, text: &str, max_chars: usize) -> Result<(), String> {
    let char_count = text.chars().count();
    if char_count > max_chars {
        return Err(format!("{label} must be {max_chars} characters or fewer."));
    }
    if text
        .chars()
        .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\r' | '\t'))
    {
        return Err(format!("{label} cannot contain control characters."));
    }

    Ok(())
}

/// Execute a Rhai script expression.
#[tauri::command]
pub async fn execute_script(state: State<'_, AppState>, script: String) -> Result<String, String> {
    validate_script_text("Script", &script, SCRIPT_MAX_TEXT_CHARS)?;
    let se = state.script_engine.read().await;
    let result = se.execute(&script).map_err(|e| e.to_string())?;
    Ok(format!("{:?}", result))
}

/// Evaluate a condition expression.
#[tauri::command]
pub async fn evaluate_condition(
    state: State<'_, AppState>,
    condition: String,
) -> Result<bool, String> {
    validate_condition_source(&condition).map_err(|e| e.to_string())?;
    let se = state.script_engine.read().await;
    se.evaluate_condition(&condition).map_err(|e| e.to_string())
}

/// Parse a DSL script into commands (WebGAL-style format).
#[tauri::command]
pub async fn parse_script(script: String) -> Result<Vec<serde_json::Value>, String> {
    validate_script_text("DSL script", &script, MAX_DSL_SCRIPT_TEXT_CHARS)?;
    use llm_game::script::ScriptParser;
    let commands = ScriptParser::parse(&script);
    let json: Vec<serde_json::Value> = commands
        .into_iter()
        .map(|cmd| serde_json::to_value(cmd).unwrap_or_default())
        .collect();
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_inputs_reject_control_characters() {
        assert!(validate_script_text("Script", "setFlag(\"ok\", true)\u{0007}", 100).is_err());
    }

    #[test]
    fn script_inputs_limit_large_payloads() {
        assert!(validate_script_text("Script", "x".repeat(101).as_str(), 100).is_err());
    }

    #[test]
    fn condition_inputs_use_shared_limits() {
        assert!(validate_condition_source(&"true".repeat(501)).is_err());
    }

    #[test]
    fn script_inputs_allow_multiline_text() {
        assert!(validate_script_text("Script", "let x = 1;\nlet y = 2;\tx + y", 100).is_ok());
    }
}
