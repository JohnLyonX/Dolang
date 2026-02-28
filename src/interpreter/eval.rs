// Expression evaluation - evaluates AST expressions to values.
use crate::ast::{Expr, FnDeclStmt};
use crate::error::Error;
use crate::token::Type;

use super::env::{generate_fn_name, list_get, list_len, serialize_list, to_bool, Env, FnEnv};

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
        Expr::IndexAccess(idx) => {
            // Evaluate the object (must be a list) - use false to get the variable's value
            let obj_val = eval_expr(&idx.object, env, fns, w, false)?;
            // Evaluate the index
            let idx_val = eval_expr(&idx.index, env, fns, w, false)?;

            // Parse index as integer
            let idx = idx_val.parse::<usize>().ok()?;

            // Get element from list
            match list_get(&obj_val, idx) {
                Some(v) => Some(v),
                None => {
                    // Check if it's a valid list but out of bounds
                    if obj_val.starts_with("__LIST__:") {
                        let len = list_len(&obj_val).unwrap_or(0);
                        return None; // Will be caught by caller with proper error
                    }
                    // Not a list at all
                    None
                }
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
