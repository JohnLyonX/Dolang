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
    List,
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
    // List values are serialized with a special prefix
    if value.starts_with(LIST_PREFIX) {
        return ValueType::List;
    }
    ValueType::String
}

/// Convert string to bool
pub fn to_bool(value: &str) -> bool {
    value == "true"
}

/// List prefix for serialization
const LIST_PREFIX: &str = "__LST__:";
const LIST_ESCAPE: &str = "__X__";

/// Serialize a list of values to a string using length-prefixed format
/// Format: __LST__:len1:value1:len2:value2:...
pub fn serialize_list(values: &[String]) -> String {
    let mut result = LIST_PREFIX.to_string();
    for v in values {
        // Escape: replace __X__ with __X__X__ (escape escape sequence)
        // and replace __LST__: with __X__LST__: (escape prefix)
        let escaped = v.replace(LIST_ESCAPE, &format!("{}X__", LIST_ESCAPE))
                       .replace(LIST_PREFIX, &format!("{}LST__:", LIST_ESCAPE));
        // Length-prefix each element
        result.push_str(&escaped.len().to_string());
        result.push(':');
        result.push_str(&escaped);
        result.push(':');
    }
    result
}

/// Deserialize a string to a list of values
/// Format: __LST__:len1:value1:len2:value2:...
pub fn deserialize_list(value: &str) -> Option<Vec<String>> {
    if !value.starts_with(LIST_PREFIX) {
        return None;
    }
    let content = &value[LIST_PREFIX.len()..];
    if content.is_empty() {
        return Some(Vec::new());
    }

    let mut results = Vec::new();
    let mut remaining = content;

    while !remaining.is_empty() {
        // Find the colon after the length
        if let Some(colon_pos) = remaining.find(':') {
            let len_str = &remaining[..colon_pos];
            let len = len_str.parse::<usize>().ok()?;
            let after_len = &remaining[colon_pos + 1..];

            // Check if we have enough content
            if after_len.len() < len {
                return None;
            }

            let element = &after_len[..len];
            // Unescape: replace __X__X__ with __X__ (unescape escape sequence)
            // and replace __X__LST__: with __LST__: (unescape prefix)
            let unescaped = element.replace(&format!("{}X__", LIST_ESCAPE), LIST_ESCAPE)
                                   .replace(&format!("{}LST__:", LIST_ESCAPE), LIST_PREFIX);
            results.push(unescaped);

            // Move past this element (length + colon + element)
            remaining = &after_len[len..];
            if !remaining.is_empty() && remaining.starts_with(':') {
                remaining = &remaining[1..];
            }
        } else {
            break;
        }
    }

    Some(results)
}

/// Get the length of a list
pub fn list_len(value: &str) -> Option<usize> {
    deserialize_list(value).map(|list| list.len())
}

/// Get an element from a list by index
pub fn list_get(value: &str, index: usize) -> Option<String> {
    let list = deserialize_list(value)?;
    if index >= list.len() {
        None
    } else {
        Some(list[index].clone())
    }
}

/// Set an element in a list by index (returns new serialized list)
pub fn list_set(value: &str, index: usize, new_value: String) -> Option<String> {
    let mut list = deserialize_list(value)?;
    if index >= list.len() {
        return None;
    }
    list[index] = new_value;
    Some(serialize_list(&list))
}
