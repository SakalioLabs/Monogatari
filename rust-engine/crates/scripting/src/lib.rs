//! # LLM Galgame Engine - Scripting
//!
//! Rhai-based scripting engine for dialogue triggers and game logic.
//!
//! ## Features
//!
//! - Variable management (get/set)
//! - Flag management (setFlag/hasFlag)
//! - Random number generation
//! - Math operations
//! - String manipulation
//! - Logging

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use rhai::{Dynamic, Engine, Scope};
use tracing::debug;

pub use llm_core::{
    normalize_script_state_key, normalize_script_state_map, SCRIPT_STATE_KEY_MAX_CHARS,
};
use llm_core::{EngineError, Result};

pub const SCRIPT_MAX_TEXT_CHARS: usize = 20_000;
pub const SCRIPT_MAX_CONDITION_CHARS: usize = 2_000;
pub const SCRIPT_MAX_OPERATIONS: u64 = 100_000;
pub const SCRIPT_MAX_CALL_LEVELS: usize = 32;
pub const SCRIPT_MAX_EXPR_DEPTH: usize = 64;
pub const SCRIPT_MAX_FUNCTION_EXPR_DEPTH: usize = 32;
pub const SCRIPT_MAX_VARIABLES: usize = 512;
pub const SCRIPT_MAX_FUNCTIONS: usize = 128;

pub fn validate_script_source(script: &str) -> Result<()> {
    validate_labeled_source("Script", script, SCRIPT_MAX_TEXT_CHARS)
}

pub fn validate_condition_source(condition: &str) -> Result<()> {
    validate_labeled_source("Condition", condition, SCRIPT_MAX_CONDITION_CHARS)
}

fn validate_labeled_source(label: &str, source: &str, max_chars: usize) -> Result<()> {
    let char_count = source.chars().count();
    if char_count > max_chars {
        return Err(EngineError::script(
            format!("{label} must be {max_chars} characters or fewer."),
            0,
            0,
        ));
    }
    if source
        .chars()
        .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\r' | '\t'))
    {
        return Err(EngineError::script(
            format!("{label} cannot contain control characters."),
            0,
            0,
        ));
    }

    Ok(())
}

fn rhai_state_key_error(error: EngineError) -> Box<rhai::EvalAltResult> {
    error.to_string().into()
}

/// A scripting engine powered by Rhai for game logic and dialogue triggers.
pub struct ScriptEngine {
    engine: Engine,
    condition_engine: Engine,
    variables: Arc<RwLock<HashMap<String, Dynamic>>>,
    flags: Arc<RwLock<HashMap<String, bool>>>,
}

impl ScriptEngine {
    /// Create a new script engine with default game functions registered.
    pub fn new() -> Self {
        let variables: Arc<RwLock<HashMap<String, Dynamic>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let flags: Arc<RwLock<HashMap<String, bool>>> = Arc::new(RwLock::new(HashMap::new()));

        let mut engine = Engine::new();
        configure_engine_limits(&mut engine);
        register_state_read_functions(&mut engine, Arc::clone(&variables), Arc::clone(&flags));
        register_state_write_functions(&mut engine, Arc::clone(&variables), Arc::clone(&flags));
        register_common_functions(&mut engine);

        let mut condition_engine = Engine::new();
        configure_engine_limits(&mut condition_engine);
        register_state_read_functions(
            &mut condition_engine,
            Arc::clone(&variables),
            Arc::clone(&flags),
        );
        register_common_functions(&mut condition_engine);

        Self {
            engine,
            condition_engine,
            variables,
            flags,
        }
    }

    /// Execute a script expression.
    pub fn execute(&self, script: &str) -> Result<Dynamic> {
        validate_script_source(script)?;
        let mut scope = Scope::new();
        let result = self
            .engine
            .eval_with_scope::<Dynamic>(&mut scope, script)
            .map_err(|e| llm_core::EngineError::script(format!("Script error: {e}"), 0, 0))?;
        Ok(result)
    }

    /// Evaluate a condition expression and return the boolean result.
    pub fn evaluate_condition(&self, condition: &str) -> Result<bool> {
        self.evaluate_condition_with_scope_variables(condition, std::iter::empty())
    }

    /// Evaluate a condition expression with temporary read-only scope variables.
    pub fn evaluate_condition_with_scope_variables<I>(
        &self,
        condition: &str,
        scope_variables: I,
    ) -> Result<bool>
    where
        I: IntoIterator<Item = (String, Dynamic)>,
    {
        validate_condition_source(condition)?;
        let mut scope = Scope::new();
        for (name, value) in scope_variables {
            scope.push_dynamic(name, value);
        }
        let result = self
            .condition_engine
            .eval_with_scope::<Dynamic>(&mut scope, condition)
            .map_err(|e| llm_core::EngineError::script(format!("Condition error: {e}"), 0, 0))?;
        match result.as_bool() {
            Ok(b) => Ok(b),
            Err(_) => Ok(false),
        }
    }

    /// Set a game variable.
    pub fn set_variable(&self, name: &str, value: Dynamic) -> Result<()> {
        let key = normalize_script_state_key(name)?;
        let mut vars = self.variables.write().unwrap();
        vars.insert(key, value);
        Ok(())
    }

    /// Get a game variable.
    pub fn get_variable(&self, name: &str) -> Option<Dynamic> {
        let key = normalize_script_state_key(name).ok()?;
        let vars = self.variables.read().unwrap();
        vars.get(&key).cloned()
    }

    /// Set a game flag.
    pub fn set_flag(&self, name: &str, value: bool) -> Result<()> {
        let key = normalize_script_state_key(name)?;
        let mut flags = self.flags.write().unwrap();
        flags.insert(key, value);
        Ok(())
    }

    /// Check if a game flag is set.
    pub fn has_flag(&self, name: &str) -> bool {
        let Ok(key) = normalize_script_state_key(name) else {
            return false;
        };
        let flags = self.flags.read().unwrap();
        flags.get(&key).copied().unwrap_or(false)
    }

    /// Get all variables as a HashMap (for saving).
    pub fn all_variables(&self) -> HashMap<String, Dynamic> {
        let vars = self.variables.read().unwrap();
        vars.clone()
    }

    /// Get all flags as a HashMap (for saving).
    pub fn all_flags(&self) -> HashMap<String, bool> {
        let flags = self.flags.read().unwrap();
        flags.clone()
    }

    /// Load variables and flags from save data.
    pub fn load_state(
        &self,
        variables: HashMap<String, Dynamic>,
        flags: HashMap<String, bool>,
    ) -> Result<()> {
        let variables = normalize_script_state_map(variables)?;
        let flags = normalize_script_state_map(flags)?;

        let mut vars = self.variables.write().unwrap();
        *vars = variables;
        let mut flgs = self.flags.write().unwrap();
        *flgs = flags;
        Ok(())
    }

    /// Load JSON-compatible variables and flags into the script runtime.
    pub fn load_json_state(
        &self,
        variables: HashMap<String, serde_json::Value>,
        flags: HashMap<String, bool>,
    ) -> Result<()> {
        let variables = variables
            .into_iter()
            .map(|(name, value)| {
                rhai::serde::to_dynamic(value)
                    .map(|value| (name, value))
                    .map_err(|error| {
                        EngineError::script(format!("Invalid script variable: {error}"), 0, 0)
                    })
            })
            .collect::<Result<HashMap<_, _>>>()?;
        self.load_state(variables, flags)
    }

    /// Export script variables and flags as JSON-compatible state.
    pub fn json_state(
        &self,
    ) -> Result<(HashMap<String, serde_json::Value>, HashMap<String, bool>)> {
        let variables = self
            .all_variables()
            .into_iter()
            .map(|(name, value)| {
                rhai::serde::from_dynamic::<serde_json::Value>(&value)
                    .map(|value| (name, value))
                    .map_err(|error| {
                        EngineError::script(format!("Invalid script result: {error}"), 0, 0)
                    })
            })
            .collect::<Result<HashMap<_, _>>>()?;
        Ok((variables, self.all_flags()))
    }

    /// Register a custom function.
    pub fn register_fn_dynamic(
        &mut self,
        name: &str,
        func: impl Fn(Dynamic) -> Dynamic + Send + Sync + 'static,
    ) {
        self.engine.register_fn(name, func);
    }
}

fn configure_engine_limits(engine: &mut Engine) {
    engine
        .set_max_operations(SCRIPT_MAX_OPERATIONS)
        .set_max_call_levels(SCRIPT_MAX_CALL_LEVELS)
        .set_max_expr_depths(SCRIPT_MAX_EXPR_DEPTH, SCRIPT_MAX_FUNCTION_EXPR_DEPTH)
        .set_max_variables(SCRIPT_MAX_VARIABLES)
        .set_max_functions(SCRIPT_MAX_FUNCTIONS)
        .set_max_modules(0);
}

fn register_state_read_functions(
    engine: &mut Engine,
    variables: Arc<RwLock<HashMap<String, Dynamic>>>,
    flags: Arc<RwLock<HashMap<String, bool>>>,
) {
    engine.register_fn(
        "getVariable",
        move |name: &str| -> std::result::Result<Dynamic, Box<rhai::EvalAltResult>> {
            let key = normalize_script_state_key(name).map_err(rhai_state_key_error)?;
            let vars = variables.read().unwrap();
            Ok(vars.get(&key).cloned().unwrap_or(Dynamic::UNIT))
        },
    );

    engine.register_fn(
        "hasFlag",
        move |name: &str| -> std::result::Result<bool, Box<rhai::EvalAltResult>> {
            let key = normalize_script_state_key(name).map_err(rhai_state_key_error)?;
            let flags = flags.read().unwrap();
            Ok(flags.get(&key).copied().unwrap_or(false))
        },
    );
}

fn register_state_write_functions(
    engine: &mut Engine,
    variables: Arc<RwLock<HashMap<String, Dynamic>>>,
    flags: Arc<RwLock<HashMap<String, bool>>>,
) {
    engine.register_fn(
        "setVariable",
        move |name: &str, value: Dynamic| -> std::result::Result<(), Box<rhai::EvalAltResult>> {
            let key = normalize_script_state_key(name).map_err(rhai_state_key_error)?;
            let mut vars = variables.write().unwrap();
            vars.insert(key, value);
            Ok(())
        },
    );

    engine.register_fn(
        "setFlag",
        move |name: &str, value: bool| -> std::result::Result<(), Box<rhai::EvalAltResult>> {
            let key = normalize_script_state_key(name).map_err(rhai_state_key_error)?;
            let mut flags = flags.write().unwrap();
            flags.insert(key, value);
            Ok(())
        },
    );
}

fn register_common_functions(engine: &mut Engine) {
    // Register utility functions
    engine.register_fn("log", |message: &str| {
        debug!("[Script] {}", message);
    });

    engine.register_fn("random_int", |min: i64, max: i64| -> i64 {
        use rand::Rng;
        rand::thread_rng().gen_range(min..=max)
    });

    // Register math functions
    engine.register_fn("abs", |x: i64| -> i64 { x.abs() });
    engine.register_fn("min", |a: i64, b: i64| -> i64 { a.min(b) });
    engine.register_fn("max", |a: i64, b: i64| -> i64 { a.max(b) });
    engine.register_fn("clamp", |x: i64, min: i64, max: i64| -> i64 {
        x.clamp(min, max)
    });

    // Register string functions
    engine.register_fn("len", |s: &str| -> i64 { s.len() as i64 });
    engine.register_fn("contains", |s: &str, sub: &str| -> bool { s.contains(sub) });
    engine.register_fn("starts_with", |s: &str, prefix: &str| -> bool {
        s.starts_with(prefix)
    });
    engine.register_fn("ends_with", |s: &str, suffix: &str| -> bool {
        s.ends_with(suffix)
    });

    // Register comparison helpers
    engine.register_fn("eq", |a: &str, b: &str| -> bool { a == b });
    engine.register_fn("ne", |a: &str, b: &str| -> bool { a != b });
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get_variable() {
        let engine = ScriptEngine::new();
        engine.set_variable("score", Dynamic::from(42i64)).unwrap();
        let val = engine.get_variable("score").unwrap();
        assert_eq!(val.as_int().unwrap(), 42);
    }

    #[test]
    fn test_set_has_flag() {
        let engine = ScriptEngine::new();
        assert!(!engine.has_flag("test"));
        engine.set_flag("test", true).unwrap();
        assert!(engine.has_flag("test"));
    }

    #[test]
    fn test_execute_condition() {
        let engine = ScriptEngine::new();
        engine.set_flag("met_sakura", true).unwrap();
        assert!(engine
            .evaluate_condition("hasFlag(\"met_sakura\")")
            .unwrap());
        assert!(!engine
            .evaluate_condition("hasFlag(\"met_unknown\")")
            .unwrap());
    }

    #[test]
    fn condition_engine_can_read_but_not_mutate_state() {
        let engine = ScriptEngine::new();
        engine.set_flag("met_sakura", true).unwrap();
        engine.set_variable("score", Dynamic::from(75i64)).unwrap();

        assert!(engine
            .evaluate_condition("hasFlag(\"met_sakura\") && getVariable(\"score\") >= 75")
            .unwrap());
        assert!(engine
            .evaluate_condition("setFlag(\"condition_mutated\", true)")
            .is_err());
        assert!(engine
            .evaluate_condition("setVariable(\"score\", 100)")
            .is_err());
        assert!(!engine.has_flag("condition_mutated"));
        assert_eq!(engine.get_variable("score").unwrap().as_int().unwrap(), 75);
    }

    #[test]
    fn condition_engine_can_read_temporary_scope_variables() {
        let engine = ScriptEngine::new();
        let variables = vec![
            ("relationship".to_string(), Dynamic::from(0.75_f64)),
            ("engagement".to_string(), Dynamic::from(0.82_f64)),
            ("evaluation_count".to_string(), Dynamic::from(3_i64)),
        ];

        assert!(engine
            .evaluate_condition_with_scope_variables(
                "relationship > 0.5 && engagement >= 0.8 && evaluation_count >= 2",
                variables,
            )
            .unwrap());
        assert!(engine.get_variable("relationship").is_none());
    }

    #[test]
    fn condition_engine_rejects_oversized_conditions() {
        let engine = ScriptEngine::new();
        let condition = "true".repeat(SCRIPT_MAX_CONDITION_CHARS + 1);

        assert!(engine.evaluate_condition(&condition).is_err());
    }

    #[test]
    fn condition_engine_rejects_control_characters() {
        let engine = ScriptEngine::new();

        assert!(engine.evaluate_condition("true\u{0007}").is_err());
    }

    #[test]
    fn direct_scripts_keep_state_mutation_functions() {
        let engine = ScriptEngine::new();

        let _ = engine
            .execute("setFlag(\"script_mutated\", true); setVariable(\"score\", 100);")
            .unwrap();

        assert!(engine.has_flag("script_mutated"));
        assert_eq!(engine.get_variable("score").unwrap().as_int().unwrap(), 100);
    }

    #[test]
    fn test_math_functions() {
        let engine = ScriptEngine::new();

        // Test abs
        let result = engine.execute("abs(-5)").unwrap();
        assert_eq!(result.as_int().unwrap(), 5);

        // Test min/max
        let result = engine.execute("min(3, 7)").unwrap();
        assert_eq!(result.as_int().unwrap(), 3);

        let result = engine.execute("max(3, 7)").unwrap();
        assert_eq!(result.as_int().unwrap(), 7);

        // Test clamp
        let result = engine.execute("clamp(5, 1, 10)").unwrap();
        assert_eq!(result.as_int().unwrap(), 5);

        let result = engine.execute("clamp(0, 1, 10)").unwrap();
        assert_eq!(result.as_int().unwrap(), 1);
    }

    #[test]
    fn test_string_functions() {
        let engine = ScriptEngine::new();

        // Test len
        let result = engine.execute("len(\"hello\")").unwrap();
        assert_eq!(result.as_int().unwrap(), 5);

        // Test contains
        let result = engine
            .execute("contains(\"hello world\", \"world\")")
            .unwrap();
        assert!(result.as_bool().unwrap());

        let result = engine
            .execute("contains(\"hello world\", \"xyz\")")
            .unwrap();
        assert!(!result.as_bool().unwrap());

        // Test starts_with
        let result = engine.execute("starts_with(\"hello\", \"hel\")").unwrap();
        assert!(result.as_bool().unwrap());

        // Test ends_with
        let result = engine.execute("ends_with(\"hello\", \"llo\")").unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_comparison_functions() {
        let engine = ScriptEngine::new();

        // Test eq
        let result = engine.execute("eq(\"test\", \"test\")").unwrap();
        assert!(result.as_bool().unwrap());

        let result = engine.execute("eq(\"test\", \"other\")").unwrap();
        assert!(!result.as_bool().unwrap());

        // Test ne
        let result = engine.execute("ne(\"test\", \"other\")").unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_random_function() {
        let engine = ScriptEngine::new();

        // Test random_int
        let result = engine.execute("random_int(1, 100)").unwrap();
        let val = result.as_int().unwrap();
        assert!((1..=100).contains(&val));
    }

    #[test]
    fn test_complex_script() {
        let engine = ScriptEngine::new();

        // Set some variables
        engine
            .set_variable("health", Dynamic::from(100i64))
            .unwrap();
        engine.set_variable("damage", Dynamic::from(25i64)).unwrap();

        // Execute a complex script
        let result = engine
            .execute("let h = getVariable(\"health\"); let d = getVariable(\"damage\"); h - d")
            .unwrap();
        assert_eq!(result.as_int().unwrap(), 75);
    }

    #[test]
    fn script_engine_limits_runaway_loops() {
        let engine = ScriptEngine::new();

        assert!(engine.execute("while true { }").is_err());
    }

    #[test]
    fn script_engine_limits_recursive_calls() {
        let engine = ScriptEngine::new();

        assert!(engine
            .execute("fn recurse() { recurse(); } recurse();")
            .is_err());
    }

    #[test]
    fn script_engine_rejects_control_characters_before_execution() {
        let engine = ScriptEngine::new();

        assert!(engine.execute("setFlag(\"ok\", true)\u{0007}").is_err());
    }

    #[test]
    fn script_engine_rejects_oversized_source_before_execution() {
        let engine = ScriptEngine::new();
        let script = "x".repeat(SCRIPT_MAX_TEXT_CHARS + 1);

        assert!(engine.execute(&script).is_err());
    }

    #[test]
    fn script_engine_normalizes_portable_state_keys() {
        let engine = ScriptEngine::new();

        engine
            .set_variable(" chapter_1.score ", Dynamic::from(42i64))
            .unwrap();
        engine.set_flag(" chapter_1.passed ", true).unwrap();

        assert_eq!(
            engine
                .get_variable("chapter_1.score")
                .unwrap()
                .as_int()
                .unwrap(),
            42
        );
        assert!(engine.has_flag("chapter_1.passed"));
    }

    #[test]
    fn script_engine_rejects_invalid_variable_names() {
        let engine = ScriptEngine::new();

        assert!(engine.set_variable("bad/key", Dynamic::from(1i64)).is_err());
        assert!(engine.execute("setVariable(\"bad/key\", 1)").is_err());
        assert!(engine.execute("getVariable(\"bad key\")").is_err());
    }

    #[test]
    fn script_engine_rejects_invalid_flag_names() {
        let engine = ScriptEngine::new();

        assert!(engine.set_flag("bad:key", true).is_err());
        assert!(engine.execute("setFlag(\"bad:key\", true)").is_err());
        assert!(engine.execute("hasFlag(\"bad/key\")").is_err());
    }

    #[test]
    fn load_state_rejects_invalid_keys() {
        let engine = ScriptEngine::new();
        let variables = HashMap::from([("bad key".to_string(), Dynamic::from(1i64))]);
        let flags = HashMap::new();

        assert!(engine.load_state(variables, flags).is_err());
    }

    #[test]
    fn json_state_round_trips_condition_and_script_values() {
        let engine = ScriptEngine::new();
        engine
            .load_json_state(
                HashMap::from([
                    ("score".to_string(), serde_json::json!(2)),
                    ("route".to_string(), serde_json::json!("aoi")),
                ]),
                HashMap::from([("started".to_string(), true)]),
            )
            .unwrap();

        assert!(engine
            .evaluate_condition("hasFlag(\"started\") && getVariable(\"score\") == 2")
            .unwrap());
        let _ = engine
            .execute("setVariable(\"score\", 3); setFlag(\"finished\", true);")
            .unwrap();
        let (variables, flags) = engine.json_state().unwrap();
        assert_eq!(variables.get("score"), Some(&serde_json::json!(3)));
        assert_eq!(variables.get("route"), Some(&serde_json::json!("aoi")));
        assert_eq!(flags.get("finished"), Some(&true));
    }
}
