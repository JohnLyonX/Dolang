// Environment management - types for variables and functions.
use super::value::DolangValue;
use crate::ast::FnDeclStmt;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Represents the type of a value (for type checking)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    Dynamic, // No type annotation, can change freely
    Int,
    Float,
    String,
    Bool,
    List,
    Map,
    File,
    Json,
    Response,
}

/// Get the ValueType from a DolangValue
pub fn get_value_type(val: &DolangValue) -> ValueType {
    match val {
        DolangValue::Int(_) => ValueType::Int,
        DolangValue::Float(_) => ValueType::Float,
        DolangValue::Str(_) => ValueType::String,
        DolangValue::Bool(_) => ValueType::Bool,
        DolangValue::List(_) => ValueType::List,
        DolangValue::Map(_) => ValueType::Map,
        DolangValue::Function { .. } => ValueType::Dynamic,
        DolangValue::File { .. } => ValueType::File,
        DolangValue::Json(_) => ValueType::Json,
        DolangValue::Html(_) => ValueType::Dynamic,
        DolangValue::Response { .. } => ValueType::Response,
        DolangValue::ModuleProxy { .. } => ValueType::Dynamic,
        DolangValue::TypedInstance { .. } => ValueType::Dynamic,
        DolangValue::Connection { .. } => ValueType::Dynamic,
        DolangValue::Null => ValueType::Dynamic,
    }
}

/// Variable environment: name -> DolangValue
pub type Env = HashMap<String, DolangValue>;

/// Type environment: tracks declared types for variables with type annotations
/// This is used to enforce type checking on assignment
/// Key: variable name, Value: declared type (if any)
pub type TypeEnv = HashMap<String, ValueType>;

/// Constant environment: tracks which variables are constants
/// Key: variable name, Value: true if constant
pub type ConstEnv = HashMap<String, bool>;

#[derive(Debug, Clone)]
pub struct RuntimeFn {
    pub decl: FnDeclStmt,
    pub source_file: Option<String>,
}

/// Function environment: name -> runtime function metadata
pub type FnEnv = HashMap<String, RuntimeFn>;

/// Counter for generating unique anonymous function names
static FN_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Generate a unique name for anonymous function
pub fn generate_fn_name() -> String {
    let n = FN_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("__anon_fn_{}__", n)
}

/// Convert type annotation string to ValueType
/// Returns None if the type annotation is invalid
pub fn parse_type_annotation(type_str: &str) -> Option<ValueType> {
    match type_str {
        "Int" => Some(ValueType::Int),
        "Float" => Some(ValueType::Float),
        "String" => Some(ValueType::String),
        "Bool" => Some(ValueType::Bool),
        "List" => Some(ValueType::List),
        "Map" => Some(ValueType::Map),
        _ => None,
    }
}

/// Get the type name for display
pub fn type_name(vt: &ValueType) -> &'static str {
    match vt {
        ValueType::Dynamic => "Dynamic",
        ValueType::Int => "Int",
        ValueType::Float => "Float",
        ValueType::String => "String",
        ValueType::Bool => "Bool",
        ValueType::List => "List",
        ValueType::Map => "Map",
        ValueType::File => "File",
        ValueType::Json => "Json",
        ValueType::Response => "Response",
    }
}
