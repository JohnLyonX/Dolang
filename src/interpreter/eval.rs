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
        Expr::StringLiteral(s) => Some(s.value.as_ref().to_string()),
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
                        let len = map_len(&obj_val).unwrap_or(0);
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
                    _ => {
                        return Some(format!("[ERROR] list has no method '{}'", call.method));
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
                        if let Some(removed) = map.remove(&arg_vals[0]) {
                            // Return a new map without the key
                            return Some(super::env::serialize_map(&map));
                        } else {
                            return Some("[ERROR] key not found".to_string());
                        }
                    }
                    _ => {
                        return Some(format!("[ERROR] map has no method '{}'", call.method));
                    }
                }
            } else {
                // String methods - strings are any value that doesn't start with special prefixes
                // and wasn't parsed as Number or Bool
                let s = &obj_val;
                if !s.starts_with("__LST__:") && !s.starts_with("__MAP__:")
                    && s.parse::<f64>().is_err() && *s != "true" && *s != "false" {
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
                        _ => {
                            return Some(format!("[ERROR] string has no method '{}'", call.method));
                        }
                    }
                }

                // Try to determine the type and report error
                let type_name = if obj_val.parse::<f64>().is_ok() {
                    "Number"
                } else if obj_val == "true" || obj_val == "false" {
                    "Bool"
                } else {
                    "Unknown"
                };
                return Some(format!("[ERROR] '{}' has no method '{}'", type_name, call.method));
            }
        }
        Expr::VarLookup(v) => {
            if as_identifier {
                Some(v.name.as_ref().to_string())
            } else {
                match env.get(v.name.as_ref()) {
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
                        Some(format_number(l + r))
                    } else {
                        Some(left + &right)
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
