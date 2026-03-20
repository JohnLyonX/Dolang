use indexmap::IndexMap;

use crate::ast::{Expr, HtmlConstructor, JsonConstructor, ResConstructor};
use crate::diagnostics::codes;
use crate::error::Error;
use crate::runtime::RuntimeContext;

use super::super::env::{Env, FnEnv};
use super::super::value::DolangValue;
use super::eval_expr;

pub fn eval_json_constructor(
    json: &JsonConstructor,
    env: &Env,
    fns: &mut FnEnv,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Result<DolangValue, Error> {
    let mut map: IndexMap<String, DolangValue> = IndexMap::new();
    for (key, value_expr) in &json.entries {
        let val = eval_expr(value_expr, env, fns, context, w, false)?;
        map.insert(key.clone(), val);
    }
    Ok(DolangValue::Json(map))
}

pub fn eval_html_constructor(
    html: &HtmlConstructor,
    env: &Env,
    fns: &mut FnEnv,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Result<DolangValue, Error> {
    let val = eval_expr(&html.content, env, fns, context, w, false)?;
    Ok(DolangValue::Html(Box::new(val)))
}

pub fn eval_res_constructor(
    e: &Expr,
    res: &ResConstructor,
    env: &Env,
    fns: &mut FnEnv,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Result<DolangValue, Error> {
    let status_val = match eval_expr(&res.status, env, fns, context, w, false) {
        Ok(DolangValue::Int(n)) => n as u16,
        _ => {
            return Err(super::runtime_error(
                e,
                codes::RUNTIME_GENERIC,
                "$RES status must be an integer",
            ));
        }
    };

    let body_val = if let Some(body_expr) = &res.body {
        Some(Box::new(eval_expr(body_expr, env, fns, context, w, false)?))
    } else {
        None
    };

    Ok(DolangValue::Response {
        status: status_val,
        body: body_val,
    })
}
