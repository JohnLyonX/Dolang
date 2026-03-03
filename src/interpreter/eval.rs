// Expression evaluation - evaluates AST expressions to values.
use crate::ast::{Expr, FnDeclStmt};
use crate::error::Error;
use crate::token::Type;

use super::env::{generate_fn_name, list_get, list_len, map_get, map_len, serialize_list, serialize_map, to_bool, Env, FnEnv};

/// Smart number formatting - follows these rules:
/// 1. Integer results: no decimal point (5.0 → 5)
/// 2. Float results: max 10 significant digits
/// 3. Trailing zeros removed (0.5 not 0.500...)
/// 4. Very large/small numbers use scientific notation
/// Note: This function removes trailing .0 for display purposes
pub fn format_number(n: f64) -> String {
    if n.is_nan() {
        return "NaN".to_string();
    }
    if n.is_infinite() {
        return if n.is_sign_positive() { "inf" } else { "-inf" }.to_string();
    }

    let abs_n = n.abs();
    let digit_count = if abs_n > 0.0 { abs_n.log10().floor() as i32 + 1 } else { 0 };
    if digit_count > 12 || (abs_n > 0.0 && abs_n < 1e-7) {
        return format_scientific(n, 10);
    }

    if abs_n.fract().abs() < 1e-10 || abs_n >= 1e15 {
        return format!("{}", n.round() as i64);
    }

    format_float(n, 10)
}

fn format_float(n: f64, max_sig_digits: usize) -> String {
    let s = format!("{:.*}", max_sig_digits, n);

    if let Some(pos) = s.find('.') {
        let mut result = s.clone();
        while result.ends_with('0') && result.len() > pos + 1 {
            result.pop();
        }
        if result.ends_with('.') {
            result.pop();
        }
        return result;
    }

    s
}

/// Format float but always keep decimal point (e.g., 42.0 not 42)
/// Used for to_float() method to preserve type information
fn format_float_always(n: f64) -> String {
    if n.is_nan() {
        return "NaN".to_string();
    }
    if n.is_infinite() {
        return if n.is_sign_positive() { "inf" } else { "-inf" }.to_string();
    }

    let s = format!("{}", n);
    // Ensure decimal point is always present
    if !s.contains('.') {
        format!("{}.0", s)
    } else {
        s
    }
}

fn format_scientific(n: f64, max_sig_digits: usize) -> String {
    if n == 0.0 {
        return "0".to_string();
    }

    let abs_n = n.abs();
    let exp = (abs_n.log10().floor()) as i32;
    let mantissa = n / 10f64.powi(exp);
    let decimals = max_sig_digits - 1;
    let factor = 10f64.powi(decimals as i32);
    let rounded = (mantissa * factor).round() / factor;

    if rounded.abs() >= 10.0 {
        let mantissa = rounded / 10.0;
        return format!("{:.prec$}e{}", mantissa, exp + 1, prec = decimals - 1);
    }

    format!("{:.prec$}e{}", rounded, exp, prec = decimals)
}

/// Check for special error markers returned by eval_expr and convert to Error
pub fn check_eval_result(val: Option<String>) -> Result<Option<String>, Error> {
    match val {
        Some(v) if v == "__DIVZERO__" => Err(Error::Interpreter("division by zero".to_string())),
        Some(v) if v == "__MODZERO__" => Err(Error::Interpreter("modulo by zero".to_string())),
        Some(v) if v.starts_with("[ERROR]") => {
            let msg = v.trim_start_matches("[ERROR]");
            let msg = msg.trim_start_matches(" runtime error:");
            Err(Error::Interpreter(msg.trim_start_matches(" ").to_string()))
        }
        other => Ok(other),
    }
}

/// Evaluate an expression and return the result as a String.
pub fn eval_expr(
    e: &Expr,
    env: &Env,
    fns: &mut FnEnv,
    w: &mut dyn std::io::Write,
    as_identifier: bool,
) -> Option<String> {
    match e {
        Expr::Number(n) => Some(n.value.as_ref().to_string()),
        Expr::Char(c) => Some(c.value.as_ref().to_string()),
        Expr::Bool(b) => Some(b.value.to_string()),
        Expr::StringLiteral(s) => Some(format!("__STR__:{}", s.value.as_ref())),
        Expr::FString(fs) => {
            // Evaluate f-string: substitute each segment
            let mut result = String::new();
            for segment in &fs.segments {
                match segment {
                    crate::ast::FStringSegment::Literal(s) => {
                        result.push_str(s);
                    }
                    crate::ast::FStringSegment::Expression(expr) => {
                        if let Some(val) = eval_expr(expr, env, fns, w, false) {
                            // Convert value to string (handle Int, Float, Bool, etc.)
                            let str_val = if val == "true" || val == "false" {
                                val.clone()
                            } else if let Ok(_) = val.parse::<f64>() {
                                // Check if it's an integer or float
                                if let Ok(n) = val.parse::<f64>() {
                                    if n.fract() == 0.0 && n.is_finite() {
                                        // It's an integer
                                        format!("{}", n as i64)
                                    } else {
                                        val.clone()
                                    }
                                } else {
                                    val.clone()
                                }
                            } else if val.starts_with("__STR__:") {
                                val.trim_start_matches("__STR__:").to_string()
                            } else if val.starts_with("__LST__:") || val.starts_with("__MAP__:") {
                                val.clone()
                            } else {
                                val.clone()
                            };
                            result.push_str(&str_val);
                        } else {
                            return None;
                        }
                    }
                }
            }
            Some(format!("__STR__:{}", result))
        }
        Expr::ListLiteral(list) => {
            // Evaluate each element and serialize to list
            let mut elements: Vec<String> = Vec::new();
            for elem in &list.elements {
                if let Some(val) = eval_expr(elem, env, fns, w, false) {
                    elements.push(val);
                } else {
                    return None;
                }
            }
            Some(serialize_list(&elements))
        }
        Expr::MapLiteral(map) => {
            use indexmap::IndexMap;
            // Evaluate each value and create map
            let mut entries: IndexMap<String, String> = IndexMap::new();
            for (key, value_expr) in &map.entries {
                if let Some(val) = eval_expr(value_expr, env, fns, w, false) {
                    entries.insert(key.clone(), val);
                } else {
                    return None;
                }
            }
            Some(serialize_map(&entries))
        }
        Expr::IndexAccess(idx) => {
            // Evaluate the object - use false to get the variable's value
            let obj_val = eval_expr(&idx.object, env, fns, w, false)?;
            // Evaluate the index/key
            let idx_val = eval_expr(&idx.index, env, fns, w, false)?;

            // Check if it's a list (numeric index) or map (string key)
            if obj_val.starts_with("__LST__:") {
                // List access - parse index as integer
                let idx = match idx_val.parse::<usize>() {
                    Ok(i) => i,
                    Err(_) => return Some(format!("[ERROR] invalid list index: must be a non-negative integer")),
                };
                // Check bounds
                let len = list_len(&obj_val).unwrap_or(0);
                if idx >= len {
                    return Some(format!("[ERROR] list index {} out of bounds (length: {})", idx, len));
                }
                list_get(&obj_val, idx)
            } else if obj_val.starts_with("__MAP__:") {
                // Map access - use key directly
                match map_get(&obj_val, &idx_val) {
                    Some(val) => Some(val),
                    None => {
                        Some(format!("[ERROR] map key '{}' not found", idx_val))
                    }
                }
            } else {
                None
            }
        }
        Expr::MethodCall(call) => {
            // First, evaluate the object
            let obj_val = eval_expr(&call.object, env, fns, w, false)?;

            // Then evaluate the arguments
            let mut arg_vals: Vec<String> = Vec::new();
            for arg in &call.args {
                if let Some(val) = eval_expr(arg, env, fns, w, false) {
                    arg_vals.push(val);
                } else {
                    return None;
                }
            }

            // Dispatch to method based on object type and method name
            if obj_val.starts_with("__LST__:") {
                // List methods
                match call.method.as_str() {
                    "len" | "length" => {
                        return Some(list_len(&obj_val).unwrap_or(0).to_string());
                    }
                    "push" => {
                        // push(value) - append to list
                        if arg_vals.is_empty() {
                            return Some("[ERROR] method 'push' requires 1 argument".to_string());
                        }
                        // Deserialize, add, serialize back
                        let mut list = super::env::deserialize_list(&obj_val).unwrap_or_default();
                        list.push(arg_vals[0].clone());
                        return Some(super::env::serialize_list(&list));
                    }
                    "pop" => {
                        // pop() - remove last element
                        let mut list = super::env::deserialize_list(&obj_val).unwrap_or_default();
                        if list.is_empty() {
                            return Some("[ERROR] cannot pop from empty list".to_string());
                        }
                        let popped = list.pop().unwrap();
                        // Store modified list back would require mutation - for now return popped value
                        // Actually, we need to return the popped value, not the modified list
                        return Some(popped);
                    }
                    "contains" => {
                        // contains(value) - check if list contains value
                        if arg_vals.is_empty() {
                            return Some("[ERROR] method 'contains' requires 1 argument".to_string());
                        }
                        let list = super::env::deserialize_list(&obj_val).unwrap_or_default();
                        return Some(list.contains(&arg_vals[0]).to_string());
                    }
                    "reverse" => {
                        // reverse() - return reversed list
                        let mut list = super::env::deserialize_list(&obj_val).unwrap_or_default();
                        list.reverse();
                        return Some(super::env::serialize_list(&list));
                    }
                    "join" => {
                        // join(separator) - join list elements into string
                        let sep = if arg_vals.is_empty() {
                            "".to_string()
                        } else {
                            arg_vals[0].clone()
                        };
                        let list = super::env::deserialize_list(&obj_val).unwrap_or_default();
                        return Some(list.join(&sep));
                    }
                    "type" => {
                        return Some("List".to_string());
                    }
                    _ => {
                        return Some(format!("[ERROR] runtime error: list has no method '{}'", call.method));
                    }
                }
            } else if obj_val.starts_with("__MAP__:") {
                // Map methods
                match call.method.as_str() {
                    "len" | "length" => {
                        return Some(map_len(&obj_val).unwrap_or(0).to_string());
                    }
                    "keys" => {
                        // Return keys as a list
                        let map = super::env::deserialize_map(&obj_val).unwrap_or_default();
                        let keys: Vec<String> = map.keys().cloned().collect();
                        return Some(super::env::serialize_list(&keys));
                    }
                    "values" => {
                        // Return values as a list
                        let map = super::env::deserialize_map(&obj_val).unwrap_or_default();
                        let values: Vec<String> = map.values().cloned().collect();
                        return Some(super::env::serialize_list(&values));
                    }
                    "contains_key" => {
                        // contains_key(key) - check if key exists
                        if arg_vals.is_empty() {
                            return Some("[ERROR] method 'contains_key' requires 1 argument".to_string());
                        }
                        let map = super::env::deserialize_map(&obj_val).unwrap_or_default();
                        return Some(map.contains_key(&arg_vals[0]).to_string());
                    }
                    "remove" => {
                        // remove(key) - remove key and return value
                        if arg_vals.is_empty() {
                            return Some("[ERROR] method 'remove' requires 1 argument".to_string());
                        }
                        let mut map = super::env::deserialize_map(&obj_val).unwrap_or_default();
                        if map.swap_remove(&arg_vals[0]).is_some() {
                            // Return a new map without the key
                            return Some(super::env::serialize_map(&map));
                        } else {
                            return Some("[ERROR] key not found".to_string());
                        }
                    }
                    "type" => {
                        return Some("Map".to_string());
                    }
                    _ => {
                        return Some(format!("[ERROR] runtime error: map has no method '{}'", call.method));
                    }
                }
            } else if obj_val == "true" || obj_val == "false" {
                // Bool methods
                let b = obj_val == "true";
                match call.method.as_str() {
                    "to_str" => {
                        return Some(format!("__STR__:{}", b.to_string()));
                    }
                    "to_int" => {
                        return Some("[ERROR] runtime error: type mismatch: Bool cannot convert to Int".to_string());
                    }
                    "to_float" => {
                        return Some("[ERROR] runtime error: type mismatch: Bool cannot convert to Float".to_string());
                    }
                    "to_bool" => {
                        return Some(b.to_string());
                    }
                    "type" => {
                        return Some("Bool".to_string());
                    }
                    _ => {
                        return Some(format!("[ERROR] runtime error: Bool has no method '{}'", call.method));
                    }
                }
            } else if obj_val.parse::<f64>().is_ok() {
                // Number methods - could be Int or Float
                let num_str = &obj_val;
                // Check if it's an integer (no decimal point in original)
                let is_int = !num_str.contains('.');

                match call.method.as_str() {
                    "to_str" => {
                        return Some(format!("__STR__:{}", num_str));
                    }
                    "to_int" => {
                        // Convert to int (truncate for floats)
                        if let Ok(i) = num_str.parse::<i64>() {
                            return Some(i.to_string());
                        } else if let Ok(f) = num_str.parse::<f64>() {
                            // Try parsing as float and truncate
                            return Some((f as i64).to_string());
                        } else {
                            return Some("[ERROR] runtime error: cannot convert to Int".to_string());
                        }
                    }
                    "to_float" => {
                        if let Ok(f) = num_str.parse::<f64>() {
                            return Some(format_float_always(f));
                        } else {
                            return Some("[ERROR] runtime error: cannot convert to Float".to_string());
                        }
                    }
                    "to_bool" => {
                        // Only allow 0 or 1 for integers
                        if is_int {
                            if let Ok(i) = num_str.parse::<i64>() {
                                if i == 0 {
                                    return Some("false".to_string());
                                } else if i == 1 {
                                    return Some("true".to_string());
                                } else {
                                    return Some(format!("[ERROR] runtime error: cannot convert {} to Bool\nexpected 0 or 1", i));
                                }
                            }
                        }
                        return Some("[ERROR] runtime error: type mismatch: Float cannot convert to Bool".to_string());
                    }
                    "type" => {
                        return Some(if is_int { "Int" } else { "Float" }.to_string());
                    }
                    _ => {
                        return Some(format!("[ERROR] runtime error: Number has no method '{}'", call.method));
                    }
                }
            } else if obj_val.starts_with("__STR__:") {
                // String values from to_str() conversion
                let s = obj_val.trim_start_matches("__STR__:");
                match call.method.as_str() {
                    "len" | "length" => {
                        return Some(s.len().to_string());
                    }
                    "upper" => {
                        return Some(s.to_uppercase());
                    }
                    "lower" => {
                        return Some(s.to_lowercase());
                    }
                    "trim" => {
                        return Some(s.trim().to_string());
                    }
                    "contains" => {
                        if arg_vals.is_empty() {
                            return Some("[ERROR] method 'contains' requires 1 argument".to_string());
                        }
                        let arg = arg_vals[0].trim_start_matches("__STR__:");
                        return Some(s.contains(arg).to_string());
                    }
                    "starts_with" => {
                        if arg_vals.is_empty() {
                            return Some("[ERROR] method 'starts_with' requires 1 argument".to_string());
                        }
                        let arg = arg_vals[0].trim_start_matches("__STR__:");
                        return Some(s.starts_with(arg).to_string());
                    }
                    "ends_with" => {
                        if arg_vals.is_empty() {
                            return Some("[ERROR] method 'ends_with' requires 1 argument".to_string());
                        }
                        let arg = arg_vals[0].trim_start_matches("__STR__:");
                        return Some(s.ends_with(arg).to_string());
                    }
                    "replace" => {
                        if arg_vals.len() != 2 {
                            return Some("[ERROR] method 'replace' requires 2 arguments".to_string());
                        }
                        let old = arg_vals[0].trim_start_matches("__STR__:");
                        let new = arg_vals[1].trim_start_matches("__STR__:");
                        return Some(s.replace(old, new));
                    }
                    "split" => {
                        if arg_vals.is_empty() {
                            return Some("[ERROR] method 'split' requires 1 argument".to_string());
                        }
                        let sep = arg_vals[0].trim_start_matches("__STR__:");
                        let parts: Vec<String> = s.split(sep).map(|s| format!("__STR__:{}", s)).collect();
                        return Some(super::env::serialize_list(&parts));
                    }
                    "slice" => {
                        if arg_vals.len() != 2 {
                            return Some("[ERROR] method 'slice' requires 2 arguments".to_string());
                        }
                        if let (Ok(start), Ok(end)) = (arg_vals[0].parse::<usize>(), arg_vals[1].parse::<usize>()) {
                            let chars: Vec<char> = s.chars().collect();
                            if start < chars.len() && end <= chars.len() && start < end {
                                let slice: String = chars[start..end].iter().collect();
                                return Some(format!("__STR__:{}", slice));
                            }
                        }
                        return Some("[ERROR] invalid slice indices".to_string());
                    }
                    "to_int" => {
                        if let Ok(i) = s.parse::<i64>() {
                            return Some(i.to_string());
                        } else if let Ok(f) = s.parse::<f64>() {
                            return Some((f as i64).to_string());
                        } else {
                            return Some(format!("[ERROR] runtime error: cannot convert \"{}\" to Int", s));
                        }
                    }
                    "to_float" => {
                        if let Ok(f) = s.parse::<f64>() {
                            return Some(format_float_always(f));
                        } else {
                            return Some(format!("[ERROR] runtime error: cannot convert \"{}\" to Float", s));
                        }
                    }
                    "to_bool" => {
                        if s == "true" {
                            return Some("true".to_string());
                        } else if s == "false" {
                            return Some("false".to_string());
                        } else {
                            return Some(format!("[ERROR] runtime error: cannot convert \"{}\" to Bool\nexpected \"true\" or \"false\"", s));
                        }
                    }
                    "to_str" => {
                        return Some(s.to_string());
                    }
                    "type" => {
                        return Some("String".to_string());
                    }
                    _ => {
                        return Some(format!("[ERROR] runtime error: string has no method '{}'", call.method));
                    }
                }
            } else if !obj_val.starts_with("__LST__:") && !obj_val.starts_with("__MAP__:") {
                // String methods - strings are any value that doesn't start with special prefixes
                // and wasn't parsed as Number or Bool
                let s = &obj_val;
                // It's a string
                match call.method.as_str() {
                    "len" | "length" => {
                        return Some(s.len().to_string());
                    }
                    "upper" => {
                        return Some(s.to_uppercase());
                    }
                    "lower" => {
                        return Some(s.to_lowercase());
                    }
                    "trim" => {
                        return Some(s.trim().to_string());
                    }
                    "contains" => {
                        if arg_vals.is_empty() {
                            return Some("[ERROR] method 'contains' requires 1 argument".to_string());
                        }
                        return Some(s.contains(&arg_vals[0]).to_string());
                    }
                    "starts_with" => {
                        if arg_vals.is_empty() {
                            return Some("[ERROR] method 'starts_with' requires 1 argument".to_string());
                        }
                        return Some(s.starts_with(&arg_vals[0]).to_string());
                    }
                    "ends_with" => {
                        if arg_vals.is_empty() {
                            return Some("[ERROR] method 'ends_with' requires 1 argument".to_string());
                        }
                        return Some(s.ends_with(&arg_vals[0]).to_string());
                    }
                    "replace" => {
                        if arg_vals.len() < 2 {
                            return Some("[ERROR] method 'replace' requires 2 arguments".to_string());
                        }
                        return Some(s.replace(&arg_vals[0], &arg_vals[1]));
                    }
                    "split" => {
                        if arg_vals.is_empty() {
                            return Some("[ERROR] method 'split' requires 1 argument".to_string());
                        }
                        let parts: Vec<String> = s.split(&arg_vals[0]).map(|s| s.to_string()).collect();
                        return Some(super::env::serialize_list(&parts));
                    }
                    "slice" => {
                        if arg_vals.len() < 2 {
                            return Some("[ERROR] method 'slice' requires 2 arguments".to_string());
                        }
                        let start = arg_vals[0].parse::<usize>().ok();
                        let end = arg_vals[1].parse::<usize>().ok();
                        if let (Some(si), Some(ei)) = (start, end) {
                            if si < s.len() && ei <= s.len() && si < ei {
                                return Some(s[si..ei].to_string());
                            }
                        }
                        return Some("[ERROR] invalid slice indices".to_string());
                    }
                    "to_int" => {
                        // Try to parse as i64 first, then as f64 and truncate
                        if let Ok(i) = s.parse::<i64>() {
                            return Some(i.to_string());
                        } else if let Ok(f) = s.parse::<f64>() {
                            return Some((f as i64).to_string());
                        } else {
                            return Some(format!("[ERROR] runtime error: cannot convert \"{}\" to Int", s));
                        }
                    }
                    "to_float" => {
                        if let Ok(f) = s.parse::<f64>() {
                            return Some(format_float_always(f));
                        } else {
                            return Some(format!("[ERROR] runtime error: cannot convert \"{}\" to Float", s));
                        }
                    }
                    "to_bool" => {
                        if *s == "true" {
                            return Some("true".to_string());
                        } else if *s == "false" {
                            return Some("false".to_string());
                        } else {
                            return Some(format!("[ERROR] runtime error: cannot convert \"{}\" to Bool\nexpected \"true\" or \"false\"", s));
                        }
                    }
                    "to_str" => {
                        // String to_str returns the string as-is (no prefix needed)
                        return Some(s.to_string());
                    }
                    "type" => {
                        return Some("String".to_string());
                    }
                    _ => {
                        return Some(format!("[ERROR] runtime error: string has no method '{}'", call.method));
                    }
                }
            } else {
                // This should never happen since we've covered all types
                return Some("[ERROR] runtime error: unknown type for method call".to_string());
            }
        }
        Expr::VarLookup(v) => {
            if as_identifier {
                Some(v.name.as_ref().to_string())
            } else {
                let name = v.name.as_ref();
                // Normal variable lookup
                match env.get(name) {
                    Some(val) => Some(val.value.clone()),
                    None => None,
                }
            }
        }
        Expr::Unary(u) => {
            let right = eval_expr(&u.right, env, fns, w, as_identifier)?;
            match u.op {
                Type::Not => Some((!to_bool(&right)).to_string()),
                Type::Minus => {
                    if let Ok(num) = right.parse::<f64>() {
                        Some((-num).to_string())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        Expr::Binary(b) => {
            let left = eval_expr(&b.left, env, fns, w, as_identifier)?;
            let right = eval_expr(&b.right, env, fns, w, as_identifier)?;

            match b.op {
                Type::Plus => {
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        // Check if either operand is a Float (contains decimal point)
                        let is_float = left.contains('.') || right.contains('.');
                        let result = l + r;
                        if is_float {
                            Some(format_float_always(result))
                        } else {
                            Some(format_number(result))
                        }
                    } else {
                        // String concatenation - remove __STR__: prefix if present
                        let left_str = left.trim_start_matches("__STR__:");
                        let right_str = right.trim_start_matches("__STR__:");
                        Some(format!("__STR__:{}", left_str.to_string() + right_str))
                    }
                }
                Type::Minus => {
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        Some(format_number(l - r))
                    } else {
                        None
                    }
                }
                Type::Mul => {
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        Some(format_number(l * r))
                    } else {
                        None
                    }
                }
                Type::Div => {
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        if r != 0.0 { Some(format_number(l / r)) } else { Some("__DIVZERO__".to_string()) }
                    } else {
                        None
                    }
                }
                Type::Mod => {
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        if r != 0.0 { Some(format_number(l % r)) } else { Some("__MODZERO__".to_string()) }
                    } else {
                        None
                    }
                }
                Type::Eq  => Some((left == right).to_string()),
                Type::Ne  => Some((left != right).to_string()),
                Type::Gt  => {
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        Some((l > r).to_string())
                    } else {
                        None
                    }
                }
                Type::Lt  => {
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        Some((l < r).to_string())
                    } else {
                        None
                    }
                }
                Type::Gte => {
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        Some((l >= r).to_string())
                    } else {
                        None
                    }
                }
                Type::Lte => {
                    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                        Some((l <= r).to_string())
                    } else {
                        None
                    }
                }
                Type::And => Some((to_bool(&left) && to_bool(&right)).to_string()),
                Type::Or  => Some((to_bool(&left) || to_bool(&right)).to_string()),
                _ => None,
            }
        }
        Expr::FnLiteral(lit) => {
            let fn_name = generate_fn_name();
            let fn_decl = FnDeclStmt {
                span: lit.span,
                name: fn_name.clone(),
                params: lit.params.clone(),
                return_type: lit.return_type.clone(),
                body: lit.body.clone(),
            };
            fns.insert(fn_name.clone(), fn_decl);
            Some(fn_name)
        }
        Expr::Read(read_expr) => {
            use std::io;

            match read_expr.mode {
                crate::ast::ReadMode::Env => {
                    // ENV mode: get key from prompt expression
                    let key = if let Some(ref prompt_expr) = read_expr.prompt {
                        match eval_expr(prompt_expr, env, fns, w, false) {
                            Some(k) => k,
                            None => {
                                return Some("__STR__:[ERROR] parse error: ENV requires a key: $<<ENV(\"KEY\")".to_string());
                            }
                        }
                    } else {
                        return Some("__STR__:[ERROR] parse error: ENV requires a key: $<<ENV(\"KEY\")".to_string());
                    };

                    // Remove string prefix if present
                    let key = key.trim_start_matches("__STR__:");

                    // Check for empty key
                    if key.is_empty() {
                        return Some("__STR__:[ERROR] parse error: ENV requires a key: $<<ENV(\"KEY\")".to_string());
                    }

                    // Read environment variable
                    match std::env::var(key) {
                        Ok(val) => Some(val),
                        Err(_) => Some(format!("__STR__:[ERROR] runtime error: environment variable '{}' is not defined", key))
                    }
                }
                crate::ast::ReadMode::Line => {
                    // LINE mode: read a line from stdin

                    // If there's a prompt, print it first (without newline)
                    if let Some(ref prompt_expr) = read_expr.prompt {
                        match eval_expr(prompt_expr, env, fns, w, false) {
                            Some(prompt_val) => {
                                let prompt = prompt_val.trim_start_matches("__STR__:");
                                // Print prompt without newline
                                let _ = write!(w, "{}", prompt);
                                let _ = w.flush();
                            }
                            None => {
                                return None;
                            }
                        }
                    }

                    // Read from stdin
                    let mut input = String::new();
                    match io::stdin().read_line(&mut input) {
                        Ok(0) => {
                            // EOF reached - return error message as the value
                            Some("__STR__:[ERROR] runtime error: unexpected EOF on stdin".to_string())
                        }
                        Ok(_) => {
                            // Remove trailing newline and return
                            let input = input.trim_end_matches('\n').trim_end_matches('\r');
                            Some(input.to_string())
                        }
                        Err(_) => {
                            Some("__STR__:[ERROR] runtime error: failed to read from stdin".to_string())
                        }
                    }
                }
            }
        }
        Expr::FnCall(call) => {
            let mut arg_vals: Vec<String> = Vec::new();
            for arg in &call.args {
                match eval_expr(arg, env, fns, w, false) {
                    Some(val) => arg_vals.push(val),
                    None => {
                        if let Expr::VarLookup(_v) = arg {
                            return None;
                        }
                        return None;
                    }
                }
            }

            let fn_name = if let Some(var_val) = env.get(&call.name) {
                var_val.value.clone()
            } else {
                call.name.clone()
            };

            let fn_def = match fns.get(&fn_name) {
                Some(f) => f.clone(),
                None => return None,
            };

            match super::exec::call_fn(&fn_def, &arg_vals, fns, w) {
                Ok(Some(val)) => Some(val),
                Ok(None) => Some(String::new()),
                Err(Error::Interpreter(msg)) => {
                    return Some(format!("[FUNC_ERROR] {}", msg));
                }
                Err(_) => {
                    return Some("invalid expression".to_string());
                }
            }
        }
    }
}
