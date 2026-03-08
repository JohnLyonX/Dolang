// Expression evaluation - evaluates AST expressions to values.
use crate::ast::{Expr, FnDeclStmt};
use crate::config;
use crate::error::Error;
use crate::token::Type;

use super::env::{generate_fn_name, Env, FnEnv};
use super::value::DolangValue;

/// Smart number formatting - follows these rules:
/// 1. Integer results: no decimal point (5.0 → 5)
/// 2. Float results: max 10 significant digits
/// 3. Trailing zeros removed (0.5 not 0.500...)
/// 4. Very large/small numbers use scientific notation
///
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

/// Check for special error values returned by eval_expr and convert to Error
pub fn check_eval_result(val: Option<DolangValue>) -> Result<Option<DolangValue>, Error> {
    match val {
        Some(DolangValue::Str(s)) if s == "__DIVZERO__" => Err(Error::Interpreter("division by zero".to_string())),
        Some(DolangValue::Str(s)) if s == "__MODZERO__" => Err(Error::Interpreter("modulo by zero".to_string())),
        Some(DolangValue::Str(s)) if s.starts_with("[ERROR]") => {
            let msg = s.trim_start_matches("[ERROR]");
            let msg = msg.trim_start_matches(" runtime error:");
            Err(Error::Interpreter(msg.trim_start_matches(" ").to_string()))
        }
        other => Ok(other),
    }
}

/// Evaluate an expression and return the result as DolangValue.
pub fn eval_expr(
    e: &Expr,
    env: &Env,
    fns: &mut FnEnv,
    w: &mut dyn std::io::Write,
    as_identifier: bool,
) -> Option<DolangValue> {
    match e {
        Expr::Number(n) => {
            // Determine type based on whether the literal contains a decimal point
            // This preserves "1.0" as Float even though it's numerically equal to 1
            let is_float = n.value.contains('.') || n.value.contains('e') || n.value.contains('E');
            if let Ok(val) = n.value.parse::<f64>() {
                if is_float {
                    Some(DolangValue::Float(val))
                } else if val.fract() == 0.0 && val.is_finite() {
                    Some(DolangValue::Int(val as i64))
                } else {
                    Some(DolangValue::Float(val))
                }
            } else {
                Some(DolangValue::Str(n.value.as_ref().to_string()))
            }
        }
        Expr::Char(c) => Some(DolangValue::Str(c.value.as_ref().to_string())),
        Expr::Bool(b) => Some(DolangValue::Bool(b.value)),
        Expr::StringLiteral(s) => Some(DolangValue::Str(s.value.as_ref().to_string())),
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
                            result.push_str(&val.to_string());
                        } else {
                            return None;
                        }
                    }
                }
            }
            Some(DolangValue::Str(result))
        }
        Expr::ListLiteral(list) => {
            // Evaluate each element and create list
            let mut elements: Vec<DolangValue> = Vec::new();
            for elem in &list.elements {
                if let Some(val) = eval_expr(elem, env, fns, w, false) {
                    elements.push(val);
                } else {
                    return None;
                }
            }
            Some(DolangValue::List(elements))
        }
        Expr::MapLiteral(map) => {
            use indexmap::IndexMap;
            // Evaluate each value and create map
            let mut entries: IndexMap<String, DolangValue> = IndexMap::new();
            for (key, value_expr) in &map.entries {
                if let Some(val) = eval_expr(value_expr, env, fns, w, false) {
                    entries.insert(key.clone(), val);
                } else {
                    return None;
                }
            }
            Some(DolangValue::Map(entries))
        }
        Expr::IndexAccess(idx) => {
            // Evaluate the object
            let obj_val = eval_expr(&idx.object, env, fns, w, false)?;
            // Evaluate the index/key
            let idx_val = eval_expr(&idx.index, env, fns, w, false)?;

            // Check if it's a list or map
            match obj_val {
                DolangValue::List(list) => {
                    let idx = match idx_val {
                        DolangValue::Int(i) => i as usize,
                        DolangValue::Float(f) if f.fract() == 0.0 => f as usize,
                        _ => return Some(DolangValue::Str("[ERROR] invalid list index: must be a non-negative integer".to_string())),
                    };
                    if idx >= list.len() {
                        return Some(DolangValue::Str(format!("[ERROR] list index {} out of bounds (length: {})", idx, list.len())));
                    }
                    Some(list[idx].clone())
                }
                DolangValue::Map(map) => {
                    let key = idx_val.to_string();
                    match map.get(&key) {
                        Some(val) => Some(val.clone()),
                        None => Some(DolangValue::Str(format!("[ERROR] map key '{}' not found", key)))
                    }
                }
                _ => None
            }
        }
        Expr::MethodCall(call) => {
            // Evaluate the object to DolangValue
            let obj_val = eval_expr(&call.object, env, fns, w, false)?;

            // Handle ModuleProxy method calls
            if let DolangValue::ModuleProxy { path: _, fns: module_fns } = &obj_val {
                // This is a method call on a module proxy
                let method_name = &call.method;

                // Find the function in the module
                let fn_def = match module_fns.get(method_name) {
                    Some(f) => f.clone(),
                    None => return Some(DolangValue::Str(format!("[ERROR] module function '{}' not found", method_name))),
                };

                // Evaluate arguments
                let mut arg_vals: Vec<DolangValue> = Vec::new();
                for arg in &call.args {
                    if let Some(val) = eval_expr(arg, env, fns, w, false) {
                        arg_vals.push(val);
                    }
                }

                // Call the module function
                match super::exec::call_fn(&fn_def, &arg_vals, fns, w) {
                    Ok(Some(val)) => return Some(val),
                    Ok(None) => return Some(DolangValue::Null),
                    Err(Error::Interpreter(msg)) => {
                        return Some(DolangValue::Str(format!("[ERROR] {}", msg)));
                    }
                    Err(_) => {
                        return Some(DolangValue::Str("invalid expression".to_string()));
                    }
                }
            }

            // Evaluate arguments to DolangValue for regular method calls
            let mut arg_vals: Vec<DolangValue> = Vec::new();
            for arg in &call.args {
                if let Some(val) = eval_expr(arg, env, fns, w, false) {
                    arg_vals.push(val);
                } else {
                    return None;
                }
            }

            // Dispatch based on method mutability
            let result: Result<DolangValue, Error> = if super::builtins::is_method_mutating(&call.method) {
                // 可变方法暂不支持，返回错误
                return Some(DolangValue::Str(format!("[ERROR] runtime error: mutable method '{}' not yet supported", call.method)));
            } else {
                // 不可变方法：使用 DolangValue 版本
                super::builtins::dispatch(&obj_val, &call.method, &arg_vals)
            };

            match result {
                Ok(r) => Some(r),
                Err(e) => Some(DolangValue::Str(format!("[ERROR] runtime error: {}", e))),
            }
        }
        Expr::VarLookup(v) => {
            if as_identifier {
                Some(DolangValue::Str(v.name.as_ref().to_string()))
            } else {
                let name = v.name.as_ref();
                // Normal variable lookup - env now stores DolangValue
                env.get(name).cloned()
            }
        }
        Expr::Unary(u) => {
            let right = eval_expr(&u.right, env, fns, w, as_identifier)?;
            match u.op {
                Type::Not => Some(DolangValue::Bool(!right.is_truthy())),
                Type::Minus => {
                    match right {
                        DolangValue::Int(n) => Some(DolangValue::Int(-n)),
                        DolangValue::Float(f) => Some(DolangValue::Float(-f)),
                        _ => None
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
                    match (&left, &right) {
                        (DolangValue::Int(l), DolangValue::Int(r)) => Some(DolangValue::Int(l + r)),
                        (DolangValue::Int(l), DolangValue::Float(r)) => Some(DolangValue::Float(*l as f64 + r)),
                        (DolangValue::Float(l), DolangValue::Int(r)) => Some(DolangValue::Float(l + *r as f64)),
                        (DolangValue::Float(l), DolangValue::Float(r)) => Some(DolangValue::Float(l + r)),
                        (DolangValue::Str(l), DolangValue::Str(r)) => Some(DolangValue::Str(l.clone() + r)),
                        (DolangValue::Str(l), DolangValue::Int(r)) => Some(DolangValue::Str(format!("{}{}", l, r))),
                        (DolangValue::Int(l), DolangValue::Str(r)) => Some(DolangValue::Str(format!("{}{}", l, r))),
                        (DolangValue::Str(l), DolangValue::Float(r)) => Some(DolangValue::Str(format!("{}{}", l, r))),
                        (DolangValue::Float(l), DolangValue::Str(r)) => Some(DolangValue::Str(format!("{}{}", l, r))),
                        (DolangValue::List(l), DolangValue::List(r)) => {
                            let mut new_list = l.clone();
                            new_list.extend(r.clone());
                            Some(DolangValue::List(new_list))
                        }
                        _ => None
                    }
                }
                Type::Minus => {
                    match (&left, &right) {
                        (DolangValue::Int(l), DolangValue::Int(r)) => Some(DolangValue::Int(l - r)),
                        (DolangValue::Int(l), DolangValue::Float(r)) => Some(DolangValue::Float(*l as f64 - r)),
                        (DolangValue::Float(l), DolangValue::Int(r)) => Some(DolangValue::Float(l - *r as f64)),
                        (DolangValue::Float(l), DolangValue::Float(r)) => Some(DolangValue::Float(l - r)),
                        _ => None
                    }
                }
                Type::Mul => {
                    match (&left, &right) {
                        (DolangValue::Int(l), DolangValue::Int(r)) => Some(DolangValue::Int(l * r)),
                        (DolangValue::Int(l), DolangValue::Float(r)) => Some(DolangValue::Float(*l as f64 * r)),
                        (DolangValue::Float(l), DolangValue::Int(r)) => Some(DolangValue::Float(l * *r as f64)),
                        (DolangValue::Float(l), DolangValue::Float(r)) => Some(DolangValue::Float(l * r)),
                        (DolangValue::Str(s), DolangValue::Int(n)) => {
                            if *n >= 0 {
                                Some(DolangValue::Str(s.repeat(*n as usize)))
                            } else {
                                None
                            }
                        }
                        (DolangValue::List(l), DolangValue::Int(n)) => {
                            if *n >= 0 {
                                let mut new_list = Vec::new();
                                for _ in 0..*n as usize {
                                    new_list.extend(l.clone());
                                }
                                Some(DolangValue::List(new_list))
                            } else {
                                None
                            }
                        }
                        _ => None
                    }
                }
                Type::Div => {
                    match (&left, &right) {
                        (DolangValue::Int(l), DolangValue::Int(r)) => {
                            if *r != 0 {
                                if *l % *r == 0 {
                                    Some(DolangValue::Int(l / r))
                                } else {
                                    Some(DolangValue::Float(*l as f64 / *r as f64))
                                }
                            } else {
                                Some(DolangValue::Str("__DIVZERO__".to_string()))
                            }
                        }
                        (DolangValue::Int(l), DolangValue::Float(r)) => {
                            if *r != 0.0 {
                                Some(DolangValue::Float(*l as f64 / r))
                            } else {
                                Some(DolangValue::Str("__DIVZERO__".to_string()))
                            }
                        }
                        (DolangValue::Float(l), DolangValue::Int(r)) => {
                            if *r != 0 {
                                Some(DolangValue::Float(l / *r as f64))
                            } else {
                                Some(DolangValue::Str("__DIVZERO__".to_string()))
                            }
                        }
                        (DolangValue::Float(l), DolangValue::Float(r)) => {
                            if *r != 0.0 {
                                Some(DolangValue::Float(l / r))
                            } else {
                                Some(DolangValue::Str("__DIVZERO__".to_string()))
                            }
                        }
                        _ => None
                    }
                }
                Type::Mod => {
                    match (&left, &right) {
                        (DolangValue::Int(l), DolangValue::Int(r)) => {
                            if *r != 0 {
                                Some(DolangValue::Int(l % r))
                            } else {
                                Some(DolangValue::Str("__MODZERO__".to_string()))
                            }
                        }
                        (DolangValue::Int(l), DolangValue::Float(r)) => {
                            if *r != 0.0 {
                                Some(DolangValue::Float(*l as f64 % r))
                            } else {
                                Some(DolangValue::Str("__MODZERO__".to_string()))
                            }
                        }
                        (DolangValue::Float(l), DolangValue::Int(r)) => {
                            if *r != 0 {
                                Some(DolangValue::Float(l % *r as f64))
                            } else {
                                Some(DolangValue::Str("__MODZERO__".to_string()))
                            }
                        }
                        (DolangValue::Float(l), DolangValue::Float(r)) => {
                            if *r != 0.0 {
                                Some(DolangValue::Float(l % r))
                            } else {
                                Some(DolangValue::Str("__MODZERO__".to_string()))
                            }
                        }
                        _ => None
                    }
                }
                Type::Eq => Some(DolangValue::Bool(left == right)),
                Type::Ne => Some(DolangValue::Bool(left != right)),
                Type::Gt => {
                    match (&left, &right) {
                        (DolangValue::Int(l), DolangValue::Int(r)) => Some(DolangValue::Bool(l > r)),
                        (DolangValue::Float(l), DolangValue::Float(r)) => Some(DolangValue::Bool(l > r)),
                        (DolangValue::Int(l), DolangValue::Float(r)) => Some(DolangValue::Bool((*l as f64) > *r)),
                        (DolangValue::Float(l), DolangValue::Int(r)) => Some(DolangValue::Bool(*l > *r as f64)),
                        (DolangValue::Str(l), DolangValue::Str(r)) => Some(DolangValue::Bool(l > r)),
                        _ => None
                    }
                }
                Type::Lt => {
                    match (&left, &right) {
                        (DolangValue::Int(l), DolangValue::Int(r)) => Some(DolangValue::Bool(l < r)),
                        (DolangValue::Float(l), DolangValue::Float(r)) => Some(DolangValue::Bool(l < r)),
                        (DolangValue::Int(l), DolangValue::Float(r)) => Some(DolangValue::Bool((*l as f64) < *r)),
                        (DolangValue::Float(l), DolangValue::Int(r)) => Some(DolangValue::Bool(*l < *r as f64)),
                        (DolangValue::Str(l), DolangValue::Str(r)) => Some(DolangValue::Bool(l < r)),
                        _ => None
                    }
                }
                Type::Gte => {
                    match (&left, &right) {
                        (DolangValue::Int(l), DolangValue::Int(r)) => Some(DolangValue::Bool(l >= r)),
                        (DolangValue::Float(l), DolangValue::Float(r)) => Some(DolangValue::Bool(l >= r)),
                        (DolangValue::Int(l), DolangValue::Float(r)) => Some(DolangValue::Bool((*l as f64) >= *r)),
                        (DolangValue::Float(l), DolangValue::Int(r)) => Some(DolangValue::Bool(*l >= *r as f64)),
                        (DolangValue::Str(l), DolangValue::Str(r)) => Some(DolangValue::Bool(l >= r)),
                        _ => None
                    }
                }
                Type::Lte => {
                    match (&left, &right) {
                        (DolangValue::Int(l), DolangValue::Int(r)) => Some(DolangValue::Bool(l <= r)),
                        (DolangValue::Float(l), DolangValue::Float(r)) => Some(DolangValue::Bool(l <= r)),
                        (DolangValue::Int(l), DolangValue::Float(r)) => Some(DolangValue::Bool((*l as f64) <= *r)),
                        (DolangValue::Float(l), DolangValue::Int(r)) => Some(DolangValue::Bool(*l <= *r as f64)),
                        (DolangValue::Str(l), DolangValue::Str(r)) => Some(DolangValue::Bool(l <= r)),
                        _ => None
                    }
                }
                Type::And => Some(DolangValue::Bool(left.is_truthy() && right.is_truthy())),
                Type::Or => Some(DolangValue::Bool(left.is_truthy() || right.is_truthy())),
                _ => None,
            }
        }
        Expr::FnLiteral(lit) => {
            let fn_name = generate_fn_name();
            let fn_decl = FnDeclStmt {
                span: lit.span,
                name: fn_name.clone(),
                params: lit.params.clone(),
                variadic_param: lit.variadic_param.clone(),
                return_type: lit.return_type.clone(),
                body: lit.body.clone(),
            };
            fns.insert(fn_name.clone(), fn_decl);
            Some(DolangValue::Str(fn_name))
        }
        Expr::Read(read_expr) => {
            use std::io;

            match read_expr.mode {
                crate::ast::ReadMode::Env => {
                    // ENV mode: get key from prompt expression
                    let key = if let Some(ref prompt_expr) = read_expr.prompt {
                        match eval_expr(prompt_expr, env, fns, w, false) {
                            Some(k) => k.to_string(),
                            None => {
                                return Some(DolangValue::Str("[ERROR] parse error: ENV requires a key: $<<ENV(\"KEY\")".to_string()));
                            }
                        }
                    } else {
                        return Some(DolangValue::Str("[ERROR] parse error: ENV requires a key: $<<ENV(\"KEY\")".to_string()));
                    };

                    // Check for empty key
                    if key.is_empty() {
                        return Some(DolangValue::Str("[ERROR] parse error: ENV requires a key: $<<ENV(\"KEY\")".to_string()));
                    }

                    // Read environment variable
                    match std::env::var(&key) {
                        Ok(val) => Some(DolangValue::Str(val)),
                        Err(_) => Some(DolangValue::Str(format!("[ERROR] runtime error: environment variable '{}' is not defined", key)))
                    }
                }
                crate::ast::ReadMode::Line => {
                    // LINE mode: read a line from stdin

                    // If there's a prompt, print it first (without newline)
                    if let Some(ref prompt_expr) = read_expr.prompt {
                        match eval_expr(prompt_expr, env, fns, w, false) {
                            Some(prompt_val) => {
                                // Print prompt without newline
                                let _ = write!(w, "{}", prompt_val);
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
                            Some(DolangValue::Str("[ERROR] runtime error: unexpected EOF on stdin".to_string()))
                        }
                        Ok(_) => {
                            // Remove trailing newline and return
                            let input = input.trim_end_matches('\n').trim_end_matches('\r');
                            Some(DolangValue::Str(input.to_string()))
                        }
                        Err(_) => {
                            Some(DolangValue::Str("[ERROR] runtime error: failed to read from stdin".to_string()))
                        }
                    }
                }
            }
        }
        Expr::FileRead(file_read) => {
            // Evaluate path
            let path_val = match eval_expr(&file_read.path, env, fns, w, false) {
                Some(v) => v,
                None => return Some(DolangValue::Str("[ERROR] runtime error: file path is required".to_string())),
            };

            let path_str = match path_val {
                DolangValue::Str(s) => s,
                _ => return Some(DolangValue::Str("[ERROR] runtime error: file path must be a String".to_string())),
            };

            // Evaluate mode (optional): "LINES"
            let mode_str = if let Some(mode_expr) = &file_read.mode {
                match eval_expr(mode_expr, env, fns, w, false) {
                    Some(v) => Some(v.to_string()),
                    None => None,
                }
            } else {
                None
            };

            // Return a File object - reading is deferred to method call
            Some(DolangValue::File { path: path_str, mode: mode_str })
        }
        Expr::FileWrite(file_write) => {
            // Evaluate path
            let path_val = match eval_expr(&file_write.path, env, fns, w, false) {
                Some(v) => v,
                None => return Some(DolangValue::Str("[ERROR] runtime error: file path is required".to_string())),
            };

            let path_str = match path_val {
                DolangValue::Str(s) => s,
                _ => return Some(DolangValue::Str("[ERROR] runtime error: file path must be a String".to_string())),
            };

            // Evaluate mode (optional): "W", "A"
            let mode_str = if let Some(mode_expr) = &file_write.mode {
                match eval_expr(mode_expr, env, fns, w, false) {
                    Some(v) => Some(v.to_string()),
                    None => None,
                }
            } else {
                None
            };

            // Return a File object - writing is deferred to .content() method
            Some(DolangValue::File { path: path_str, mode: mode_str })
        }
        Expr::ConfigRead(config) => {
            // Check if in serve mode
            if !config::is_serve_mode() {
                return Some(DolangValue::Str("[ERROR] runtime error: $<<CONFIG() is only available in serve mode".to_string()));
            }

            // Get config value
            if let Some(cfg) = config::get_serve_config() {
                if let Some(value) = cfg.get(&config.key) {
                    return Some(DolangValue::Str(value.to_string()));
                }
                Some(DolangValue::Str(format!("[ERROR] runtime error: config '{}' not found", config.key)))
            } else {
                Some(DolangValue::Str("[ERROR] runtime error: config not loaded".to_string()))
            }
        }
        Expr::HdrRead(hdr) => {
            // Look up header from __headers__ in env
            // HTTP headers are case-insensitive, so we convert to lowercase
            let header_key = hdr.header_name.to_lowercase();
            if let Some(headers_val) = env.get("__headers__") {
                if let DolangValue::Json(headers) = headers_val {
                    // Try lowercase key first
                    if let Some(value) = headers.get(&header_key) {
                        return Some(value.clone());
                    }
                    // Also try the original key
                    if let Some(value) = headers.get(&hdr.header_name) {
                        return Some(value.clone());
                    }
                    Some(DolangValue::Str("".to_string()))
                } else {
                    Some(DolangValue::Str("".to_string()))
                }
            } else {
                // Not in HTTP context
                Some(DolangValue::Str("".to_string()))
            }
        }
        Expr::JsonConstructor(json) => {
            use indexmap::IndexMap;
            let mut map = IndexMap::new();
            for (key, value_expr) in &json.entries {
                if let Some(val) = eval_expr(value_expr, env, fns, w, false) {
                    map.insert(key.clone(), val);
                }
            }
            Some(DolangValue::Json(map))
        }
        Expr::HtmlConstructor(html) => {
            // Evaluate HTML content
            if let Some(val) = eval_expr(&html.content, env, fns, w, false) {
                Some(DolangValue::Html(Box::new(val)))
            } else {
                Some(DolangValue::Null)
            }
        }
        Expr::ResConstructor(res) => {
            // Evaluate status code
            let status_val = match eval_expr(&res.status, env, fns, w, false) {
                Some(DolangValue::Int(n)) => n as u16,
                _ => {
                    return Some(DolangValue::Str("[ERROR] runtime error: $RES status must be an integer".to_string()));
                }
            };

            // Evaluate body (optional)
            let body_val = if let Some(body_expr) = &res.body {
                eval_expr(body_expr, env, fns, w, false).map(Box::new)
            } else {
                None
            };

            Some(DolangValue::Response {
                status: status_val,
                body: body_val,
            })
        }
        Expr::FnCall(call) => {
            let mut arg_vals: Vec<DolangValue> = Vec::new();
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

            // Special handling for link() function
            if call.name == "link" {
                let module_path = match arg_vals.first() {
                    Some(DolangValue::Str(s)) => s.clone(),
                    _ => return Some(DolangValue::Str("[ERROR] link() requires module path".to_string())),
                };

                // Try to load the module
                let module_file = format!("{}.dol", module_path.replace('.', "/"));
                let content = match std::fs::read_to_string(&module_file) {
                    Ok(c) => c,
                    Err(_) => {
                        // Try with modules/ prefix
                        let module_with_prefix = format!("modules/{}.dol", module_path.replace('.', "/"));
                        match std::fs::read_to_string(&module_with_prefix) {
                            Ok(c) => c,
                            Err(e) => {
                                return Some(DolangValue::Str(format!("[ERROR] cannot load module '{}': {}", module_path, e)));
                            }
                        }
                    }
                };

                // Parse the module
                let module_stmts = match crate::parser::parse(&content) {
                    Ok(stmts) => stmts,
                    Err(e) => {
                        return Some(DolangValue::Str(format!("[ERROR] parse module '{}' failed: {}", module_path, e)));
                    }
                };

                // Extract functions from the module
                let mut module_fns: super::env::FnEnv = std::collections::HashMap::new();
                for stmt in module_stmts {
                    if let crate::ast::Stmt::FnDecl(fn_decl) = stmt {
                        module_fns.insert(fn_decl.name.clone(), fn_decl);
                    }
                }

                return Some(DolangValue::ModuleProxy {
                    path: module_path,
                    fns: module_fns,
                });
            }

            let fn_name = if let Some(var_val) = env.get(&call.name) {
                match var_val {
                    DolangValue::Str(s) => s.clone(),
                    _ => return None,
                }
            } else {
                call.name.clone()
            };

            let fn_def = match fns.get(&fn_name) {
                Some(f) => f.clone(),
                None => return None,
            };

            match super::exec::call_fn(&fn_def, &arg_vals, fns, w) {
                Ok(Some(val)) => Some(val),
                Ok(None) => Some(DolangValue::Null),
                Err(Error::Interpreter(msg)) => {
                    Some(DolangValue::Str(format!("[FUNC_ERROR] {}", msg)))
                }
                Err(_) => {
                    Some(DolangValue::Str("invalid expression".to_string()))
                }
            }
        }
    }
}
