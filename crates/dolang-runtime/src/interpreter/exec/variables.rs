use crate::ast::{AssignStmt, ConstDeclStmt, Expr, MethodCall, VarDeclStmt};
use crate::error::Error;
use crate::runtime::{ProgramState, RuntimeContext};

use super::super::env::{ValueType, get_value_type, parse_type_annotation, type_name};
use super::super::eval::{check_eval_result, eval_expr};
use super::super::value::DolangValue;
use super::Flow;

pub(super) fn handle_var_decl(
    stmt: &VarDeclStmt,
    state: &mut ProgramState,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Flow {
    if matches!(state.const_env.get(&stmt.name), Some(true)) {
        return Flow::Err(Error::Interpreter(format!(
            "constant '{}' is already defined",
            stmt.name
        )));
    }

    match check_eval_result(eval_expr(
        &stmt.value,
        &state.env,
        &mut state.fns,
        context,
        w,
        false,
    )) {
        Ok(val) => {
            if let Some(ref type_str) = stmt.type_annotation {
                let expected_type = match parse_type_annotation(type_str) {
                    Some(t) => t,
                    None => {
                        return Flow::Err(Error::TypeMismatch(format!(
                            "unknown type '{}', supported types are: Int, Float, String, Bool",
                            type_str
                        )));
                    }
                };
                let actual_type = get_value_type(&val);
                if actual_type != expected_type {
                    return Flow::Err(Error::TypeMismatch(format!(
                        "type error: declared type '{}' does not match value type '{}'",
                        type_str,
                        type_name(&actual_type)
                    )));
                }
                state.type_env.insert(stmt.name.clone(), expected_type);
            }
            state.env.insert(stmt.name.clone(), val);
            Flow::Normal
        }
        Err(err) => Flow::Err(err),
    }
}

pub(super) fn handle_const_decl(
    stmt: &ConstDeclStmt,
    state: &mut ProgramState,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Flow {
    if state.env.contains_key(&stmt.name) {
        return Flow::Err(Error::Interpreter(format!(
            "constant '{}' is already defined",
            stmt.name
        )));
    }

    match check_eval_result(eval_expr(
        &stmt.value,
        &state.env,
        &mut state.fns,
        context,
        w,
        false,
    )) {
        Ok(val) => {
            if let Some(ref type_str) = stmt.type_annotation {
                let expected_type = match parse_type_annotation(type_str) {
                    Some(t) => t,
                    None => {
                        return Flow::Err(Error::TypeMismatch(format!(
                            "unknown type '{}', supported types are: Int, Float, String, Bool",
                            type_str
                        )));
                    }
                };
                let actual_type = get_value_type(&val);
                if actual_type != expected_type {
                    return Flow::Err(Error::TypeMismatch(format!(
                        "type error: declared type '{}' does not match value type '{}'",
                        type_str,
                        type_name(&actual_type)
                    )));
                }
                state.type_env.insert(stmt.name.clone(), expected_type);
            }
            state.const_env.insert(stmt.name.clone(), true);
            state.env.insert(stmt.name.clone(), val);
            Flow::Normal
        }
        Err(err) => Flow::Err(err),
    }
}

pub(super) fn handle_assign_stmt(
    stmt: &AssignStmt,
    state: &mut ProgramState,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Flow {
    if let Expr::IndexAccess(idx) = &*stmt.name {
        let var_name = match eval_expr(&idx.object, &state.env, &mut state.fns, context, w, true) {
            Ok(DolangValue::Str(s)) => s,
            Ok(_) => {
                return Flow::Err(Error::InvalidAssignment(Some(
                    "variable name must be a string".to_string(),
                )));
            }
            Err(err) => return Flow::Err(err),
        };

        let idx_val = match eval_expr(&idx.index, &state.env, &mut state.fns, context, w, false) {
            Ok(val) => val,
            Err(err) => return Flow::Err(err),
        };

        let val = match check_eval_result(eval_expr(
            &stmt.value,
            &state.env,
            &mut state.fns,
            context,
            w,
            false,
        )) {
            Ok(val) => val,
            Err(err) => return Flow::Err(err),
        };

        let existing = match state.env.get(&var_name) {
            Some(existing) => existing,
            None => {
                return Flow::Err(Error::Interpreter(format!(
                    "variable '{}' not found",
                    var_name
                )));
            }
        };

        match existing {
            DolangValue::List(list) => {
                let index = match &idx_val {
                    DolangValue::Int(i) => *i as usize,
                    DolangValue::Float(f) if f.fract() == 0.0 => *f as usize,
                    _ => {
                        return Flow::Err(Error::Interpreter(
                            "list index must be an integer".to_string(),
                        ));
                    }
                };

                if index >= list.len() {
                    return Flow::Err(Error::Interpreter(format!(
                        "index out of bounds: list length is {} but index is {}",
                        list.len(),
                        index
                    )));
                }

                let mut new_list = list.clone();
                new_list[index] = val;
                state.env.insert(var_name, DolangValue::List(new_list));
            }
            DolangValue::Map(map) => {
                let key = idx_val.to_string();
                let mut new_map = map.clone();
                new_map.insert(key, val);
                state.env.insert(var_name, DolangValue::Map(new_map));
            }
            _ => {
                return Flow::Err(Error::Interpreter(format!(
                    "cannot index into type {}",
                    existing.type_name()
                )));
            }
        }

        return Flow::Normal;
    }

    let name = match eval_expr(&stmt.name, &state.env, &mut state.fns, context, w, true) {
        Ok(DolangValue::Str(s)) => s,
        Ok(_) => {
            return Flow::Err(Error::InvalidAssignment(Some(
                "variable name must be a string".to_string(),
            )));
        }
        Err(err) => return Flow::Err(err),
    };
    let val = match check_eval_result(eval_expr(
        &stmt.value,
        &state.env,
        &mut state.fns,
        context,
        w,
        false,
    )) {
        Ok(val) => val,
        Err(err) => return Flow::Err(err),
    };

    if let Some(is_const) = state.const_env.get(&name)
        && *is_const
    {
        return Flow::Err(Error::Interpreter(format!(
            "cannot reassign constant '{}'",
            name
        )));
    }

    if let Some(declared_type) = state.type_env.get(&name) {
        let actual_type = get_value_type(&val);
        if *declared_type != ValueType::Dynamic && actual_type != *declared_type {
            return Flow::Err(Error::Interpreter(format!(
                "type error: variable '{}' is declared as '{}', cannot assign '{}' value",
                name,
                type_name(declared_type),
                type_name(&actual_type)
            )));
        }
    }

    state.env.insert(name, val);
    Flow::Normal
}

pub(super) fn handle_expr_stmt(
    expr: &Expr,
    state: &mut ProgramState,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Flow {
    if let Expr::MethodCall(call) = expr {
        if super::super::builtins::is_method_mutating(&call.method) {
            return handle_mut_method_call(expr, call, state, context, w);
        }
    }
    match check_eval_result(eval_expr(
        expr,
        &state.env,
        &mut state.fns,
        context,
        w,
        false,
    )) {
        Ok(_) => Flow::Normal,
        Err(err) => Flow::Err(err),
    }
}

fn handle_mut_method_call(
    _e: &Expr,
    call: &MethodCall,
    state: &mut ProgramState,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Flow {
    let var_name = match &*call.object {
        Expr::VarLookup(v) => v.name.to_string(),
        _ => {
            return Flow::Err(Error::Interpreter(format!(
                "mutable method '{}' requires a direct variable as receiver",
                call.method
            )));
        }
    };

    let mut arg_vals = Vec::new();
    for arg in &call.args {
        match eval_expr(arg, &state.env, &mut state.fns, context, w, false) {
            Ok(v) => arg_vals.push(v),
            Err(err) => return Flow::Err(err),
        }
    }

    let receiver = match state.env.get_mut(&var_name) {
        Some(r) => r,
        None => {
            return Flow::Err(Error::Interpreter(format!(
                "variable '{}' is not defined",
                var_name
            )));
        }
    };

    match super::super::builtins::dispatch_mut(receiver, &call.method, &arg_vals) {
        Ok(_) => Flow::Normal,
        Err(err) => Flow::Err(err),
    }
}
