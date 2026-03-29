use indexmap::IndexMap;

use crate::ast::{Expr, HtmlConstructor, JsonConstructor, ResConstructor, Span, StructConstructor};
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

pub fn eval_struct_constructor(
    ctor: &StructConstructor,
    env: &Env,
    fns: &mut FnEnv,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Result<DolangValue, Error> {
    let shape = context.get_type(&ctor.type_name).cloned();
    if shape.is_none() {
        return Err(Error::Diagnostic(
            crate::diagnostics::Diagnostic::error(
                codes::RUNTIME_FIELD_NOT_FOUND,
                format!("type '{}' is not defined", ctor.type_name),
            )
            .with_span(ctor.span),
        ));
    }

    let mut fields: IndexMap<String, DolangValue> = IndexMap::new();
    for (key, value_expr) in &ctor.fields {
        let val = eval_expr(value_expr, env, fns, context, w, false)?;
        // @HIDE fields are stored with "_" prefix internally
        let stored_key = if let Some(ref s) = shape {
            if let Some(fd) = s.fields.iter().find(|f| f.name == *key) {
                if fd.hidden { format!("_{}", key) } else { key.clone() }
            } else {
                key.clone()
            }
        } else {
            key.clone()
        };
        fields.insert(stored_key, val);
    }

    Ok(DolangValue::TypedInstance {
        type_name: ctor.type_name.clone(),
        fields,
    })
}
