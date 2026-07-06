//! Script engine commands.

use tauri::State;

use crate::state::AppState;

/// Execute a Rhai script expression.
#[tauri::command]
pub async fn execute_script(state: State<'_, AppState>, script: String) -> Result<String, String> {
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
    let se = state.script_engine.read().await;
    se.evaluate_condition(&condition).map_err(|e| e.to_string())
}

/// Parse a DSL script into commands (WebGAL-style format).
#[tauri::command]
pub async fn parse_script(script: String) -> Result<Vec<serde_json::Value>, String> {
    use llm_game::script::ScriptParser;
    let commands = ScriptParser::parse(&script);
    let json: Vec<serde_json::Value> = commands
        .into_iter()
        .map(|cmd| serde_json::to_value(cmd).unwrap_or_default())
        .collect();
    Ok(json)
}
