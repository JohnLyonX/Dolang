use crate::ast::{ConfigReadExpr, Expr, ExprRead, FileReadExpr, FileWriteExpr, HdrReadExpr};
use crate::diagnostics::codes;
use crate::error::Error;
use crate::runtime::{ProgramState, RuntimeContext, intrinsics::ids};

use super::super::value::DolangValue;
use super::eval_expr;

pub fn eval_read_expr(
    e: &Expr,
    read_expr: &ExprRead,
    state: &mut ProgramState,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Result<DolangValue, Error> {
    use std::io;

    match read_expr.mode {
        crate::ast::ReadMode::Env => {
            let key = if let Some(ref prompt_expr) = read_expr.prompt {
                eval_expr(prompt_expr, state, context, w, false)?.to_string()
            } else {
                return Err(super::runtime_error(
                    e,
                    codes::RUNTIME_INVALID_EXPRESSION,
                    "ENV requires a key: $<<ENV(\"KEY\")",
                ));
            };

            if key.is_empty() {
                return Err(super::runtime_error(
                    e,
                    codes::RUNTIME_INVALID_EXPRESSION,
                    "ENV requires a key: $<<ENV(\"KEY\")",
                ));
            }

            context
                .call_intrinsic(ids::ENV_GET, &[DolangValue::Str(key)])
                .map_err(|err| super::runtime_error(e, codes::RUNTIME_GENERIC, err.to_string()))
        }
        crate::ast::ReadMode::Line => {
            if let Some(ref prompt_expr) = read_expr.prompt {
                let prompt_val = eval_expr(prompt_expr, state, context, w, false)?;
                let _ = write!(w, "{prompt_val}");
                let _ = w.flush();
            }

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) => Err(super::runtime_error(
                    e,
                    codes::RUNTIME_GENERIC,
                    "unexpected EOF on stdin",
                )),
                Ok(_) => {
                    let input = input.trim_end_matches('\n').trim_end_matches('\r');
                    Ok(DolangValue::Str(input.to_string()))
                }
                Err(_) => Err(super::runtime_error(
                    e,
                    codes::RUNTIME_GENERIC,
                    "failed to read from stdin",
                )),
            }
        }
    }
}

pub fn eval_file_read_expr(
    e: &Expr,
    file_read: &FileReadExpr,
    state: &mut ProgramState,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Result<DolangValue, Error> {
    let path_val = eval_expr(&file_read.path, state, context, w, false)?;
    let path_str = match path_val {
        DolangValue::Str(s) => s,
        _ => {
            return Err(super::runtime_error(
                e,
                codes::RUNTIME_GENERIC,
                "file path must be a String",
            ));
        }
    };

    let mode_str = if let Some(mode_expr) = &file_read.mode {
        Some(eval_expr(mode_expr, state, context, w, false)?.to_string())
    } else {
        None
    };

    Ok(DolangValue::File {
        path: path_str,
        mode: mode_str,
    })
}

pub fn eval_file_write_expr(
    e: &Expr,
    file_write: &FileWriteExpr,
    state: &mut ProgramState,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Result<DolangValue, Error> {
    let path_val = eval_expr(&file_write.path, state, context, w, false)?;
    let path_str = match path_val {
        DolangValue::Str(s) => s,
        _ => {
            return Err(super::runtime_error(
                e,
                codes::RUNTIME_GENERIC,
                "file path must be a String",
            ));
        }
    };

    let mode_str = if let Some(mode_expr) = &file_write.mode {
        Some(eval_expr(mode_expr, state, context, w, false)?.to_string())
    } else {
        None
    };

    Ok(DolangValue::File {
        path: path_str,
        mode: mode_str,
    })
}

pub fn eval_config_read_expr(
    e: &Expr,
    config: &ConfigReadExpr,
    context: &mut RuntimeContext,
) -> Result<DolangValue, Error> {
    context
        .call_intrinsic(ids::CONFIG_GET, &[DolangValue::Str(config.key.clone())])
        .map_err(|err| super::runtime_error(e, codes::RUNTIME_GENERIC, err.to_string()))
}

pub fn eval_hdr_read_expr(hdr: &HdrReadExpr, state: &ProgramState) -> Result<DolangValue, Error> {
    let header_key = hdr.header_name.to_lowercase();
    if let Some(DolangValue::Json(headers)) = state.lookup_env("__headers__") {
        if let Some(value) = headers.get(&header_key) {
            return Ok(value.clone());
        }
        if let Some(value) = headers.get(&hdr.header_name) {
            return Ok(value.clone());
        }
    }
    Ok(DolangValue::Str(String::new()))
}
