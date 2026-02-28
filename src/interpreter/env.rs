// Environment management - types for variables and functions.
use crate::ast::FnDeclStmt;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Represents the type of a value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    Number,
    String,
    Bool,
}

/// Represents a value that can be either a variable or constant
#[derive(Clone)]
pub struct VarValue {
    pub value: String,
    pub value_type: ValueType,
    pub is_const: bool,
}

/// Variable environment: name -> VarValue
pub type Env = HashMap<String, VarValue>;

/// Function environment: name -> FnDeclStmt
pub type FnEnv = HashMap<String, FnDeclStmt>;

/// Counter for generating unique anonymous function names
static FN_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Generate a unique name for anonymous function
pub fn generate_fn_name() -> String {
    let n = FN_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("__anon_fn_{}__", n)
}

/// Detect the type of a value string
pub fn detect_type(value: &str) -> ValueType {
    if value == "true" || value == "false" {
        return ValueType::Bool;
    }
    if value.parse::<f64>().is_ok() {
        return ValueType::Number;
    }
    ValueType::String
}

/// Convert string to bool
pub fn to_bool(value: &str) -> bool {
    value == "true"
}
