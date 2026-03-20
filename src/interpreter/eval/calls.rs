use crate::ast::{Expr, FnCallExpr, MethodCall};
use crate::diagnostics::codes;
use crate::error::Error;
use crate::runtime::RuntimeContext;

use super::super::env::{Env, FnEnv};
use super::super::value::DolangValue;
use super::eval_expr;

pub fn eval_method_call(
    e: &Expr,
    call: &MethodCall,
    env: &Env,
    fns: &mut FnEnv,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Result<DolangValue, Error> {
    let obj_val = eval_expr(&call.object, env, fns, context, w, false)?;

    if let DolangValue::ModuleProxy {
        path: _,
        exports,
        fns: module_fns,
    } = &obj_val
    {
        let method_name = &call.method;
        let fn_def = match exports.get(method_name) {
            Some(f) => f.clone(),
            None => {
                return Err(super::runtime_error(
                    e,
                    codes::RUNTIME_MODULE_LOAD,
                    format!("module function '{method_name}' not found"),
                ));
            }
        };

        let mut arg_vals = Vec::new();
        for arg in &call.args {
            arg_vals.push(eval_expr(arg, env, fns, context, w, false)?);
        }

        let mut module_scope_fns = fns.clone();
        module_scope_fns.extend(module_fns.clone());

        return match super::super::exec::call_fn(
            &fn_def,
            &arg_vals,
            &mut module_scope_fns,
            context,
            w,
        ) {
            Ok(Some(val)) => Ok(val),
            Ok(None) => Ok(DolangValue::Null),
            Err(err) => Err(err),
        };
    }

    let mut arg_vals = Vec::new();
    for arg in &call.args {
        arg_vals.push(eval_expr(arg, env, fns, context, w, false)?);
    }

    if super::super::builtins::is_method_mutating(&call.method) {
        return Err(super::runtime_error(
            e,
            codes::RUNTIME_GENERIC,
            format!("mutable method '{}' not yet supported", call.method),
        ));
    }

    super::super::builtins::dispatch(&obj_val, &call.method, &arg_vals, context)
}

pub fn eval_fn_call(
    e: &Expr,
    call: &FnCallExpr,
    env: &Env,
    fns: &mut FnEnv,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Result<DolangValue, Error> {
    let mut arg_vals = Vec::new();
    for arg in &call.args {
        arg_vals.push(eval_expr(arg, env, fns, context, w, false)?);
    }

    let fn_name = if let Some(var_val) = env.get(&call.name) {
        match var_val {
            DolangValue::Str(s) => s.clone(),
            _ => {
                return Err(super::runtime_error(
                    e,
                    codes::RUNTIME_INVALID_EXPRESSION,
                    format!("'{}' is not a callable function name", call.name),
                ));
            }
        }
    } else {
        call.name.clone()
    };

    let fn_def = match fns.get(&fn_name) {
        Some(f) => f.clone(),
        None => return Err(super::undefined_variable_error(e, &fn_name)),
    };

    match super::super::exec::call_fn(&fn_def, &arg_vals, fns, context, w) {
        Ok(Some(val)) => Ok(val),
        Ok(None) => Ok(DolangValue::Null),
        Err(err) => Err(err),
    }
}
