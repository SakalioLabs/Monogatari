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
pub const SCRIPT_MAX_OPERATIONS: u64 = 100_000;
pub const SCRIPT_MAX_CALL_LEVELS: usize = 32;
pub const SCRIPT_MAX_EXPR_DEPTH: usize = 64;
pub const SCRIPT_MAX_FUNCTION_EXPR_DEPTH: usize = 32;
pub const SCRIPT_MAX_VARIABLES: usize = 512;
pub const SCRIPT_MAX_FUNCTIONS: usize = 128;

pub fn validate_script_source(script: &str) -> Result<()> {
    let char_count = script.chars().count();
    if char_count > SCRIPT_MAX_TEXT_CHARS {
        return Err(EngineError::script(
            format!("Script must be {SCRIPT_MAX_TEXT_CHARS} characters or fewer."),
            0,
            0,
        ));
    }
    if script
        .chars()
        .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\r' | '\t'))
    {
        return Err(EngineError::script(
            "Script cannot contain control characters.",
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
    variables: Arc<RwLock<HashMap<String, Dynamic>>>,
    flags: Arc<RwLock<HashMap<String, bool>>>,
}

impl ScriptEngine {
    /// Create a new script engine with default game functions registered.
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine
            .set_max_operations(SCRIPT_MAX_OPERATIONS)
            .set_max_call_levels(SCRIPT_MAX_CALL_LEVELS)
            .set_max_expr_depths(SCRIPT_MAX_EXPR_DEPTH, SCRIPT_MAX_FUNCTION_EXPR_DEPTH)
            .set_max_variables(SCRIPT_MAX_VARIABLES)
            .set_max_functions(SCRIPT_MAX_FUNCTIONS)
            .set_max_modules(0);

        let variables: Arc<RwLock<HashMap<String, Dynamic>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let flags: Arc<RwLock<HashMap<String, bool>>> = Arc::new(RwLock::new(HashMap::new()));

        // Register game-specific functions
        let vars = Arc::clone(&variables);
        engine.register_fn(
            "setVariable",
            move |name: &str,
                  value: Dynamic|
                  -> std::result::Result<(), Box<rhai::EvalAltResult>> {
                let key = normalize_script_state_key(name).map_err(rhai_state_key_error)?;
                let mut vars = vars.write().unwrap();
                vars.insert(key, value);
                Ok(())
            },
        );

        let vars = Arc::clone(&variables);
        engine.register_fn(
            "getVariable",
            move |name: &str| -> std::result::Result<Dynamic, Box<rhai::EvalAltResult>> {
                let key = normalize_script_state_key(name).map_err(rhai_state_key_error)?;
                let vars = vars.read().unwrap();
                Ok(vars.get(&key).cloned().unwrap_or(Dynamic::UNIT))
            },
        );

        let flgs = Arc::clone(&flags);
        engine.register_fn(
            "setFlag",
            move |name: &str, value: bool| -> std::result::Result<(), Box<rhai::EvalAltResult>> {
                let key = normalize_script_state_key(name).map_err(rhai_state_key_error)?;
                let mut flags = flgs.write().unwrap();
                flags.insert(key, value);
                Ok(())
            },
        );

        let flgs = Arc::clone(&flags);
        engine.register_fn(
            "hasFlag",
            move |name: &str| -> std::result::Result<bool, Box<rhai::EvalAltResult>> {
                let key = normalize_script_state_key(name).map_err(rhai_state_key_error)?;
                let flags = flgs.read().unwrap();
                Ok(flags.get(&key).copied().unwrap_or(false))
            },
        );

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

        Self {
            engine,
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
        let result = self.execute(condition)?;
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

    /// Register a custom function.
    pub fn register_fn_dynamic(
        &mut self,
        name: &str,
        func: impl Fn(Dynamic) -> Dynamic + Send + Sync + 'static,
    ) {
        self.engine.register_fn(name, func);
    }
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
        assert!(val >= 1 && val <= 100);
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
}
