use std::io::Write;

use crate::ast::{FileReadStmt, FileWriteStmt, PrintStmt, ReadStmt};
use crate::error::Error;
use crate::runtime::{IntrinsicId, ProgramState, RuntimeContext};

use super::super::eval::{check_eval_result, eval_expr};
use super::super::value::DolangValue;
use super::Flow;

pub(super) fn handle_print_stmt(
    stmt: &PrintStmt,
    state: &mut ProgramState,
    context: &mut RuntimeContext,
    w: &mut dyn Write,
) -> Flow {
    let use_stderr = stmt.target == crate::ast::PrintTarget::Stderr;
    match check_eval_result(eval_expr(
        &stmt.value,
        &state.env,
        &mut state.fns,
        context,
        w,
        false,
    )) {
        Ok(val) => {
            if use_stderr {
                eprintln!("{}", val);
            } else if writeln!(w, "{}", val).is_err() {
                return Flow::Err(Error::InvalidStatement(None));
            }
            Flow::Normal
        }
        Err(err) => Flow::Err(err),
    }
}

pub(super) fn handle_read_stmt(
    stmt: &ReadStmt,
    state: &mut ProgramState,
    context: &mut RuntimeContext,
    w: &mut dyn Write,
) -> Flow {
    use std::io;

    match stmt.mode {
        crate::ast::ReadMode::Env => {
            let key_val = if let Some(ref prompt_expr) = stmt.prompt {
                match eval_expr(prompt_expr, &state.env, &mut state.fns, context, w, false) {
                    Ok(val) => val,
                    Err(err) => return Flow::Err(err),
                }
            } else {
                return Flow::Err(Error::Interpreter(
                    "ENV requires a key: $<<ENV(\"KEY\")".to_string(),
                ));
            };

            let key = key_val.to_string();
            if key.is_empty() {
                return Flow::Err(Error::Interpreter(
                    "ENV requires a key: $<<ENV(\"KEY\")".to_string(),
                ));
            }

            match context.call_intrinsic(IntrinsicId::EnvGet, &[DolangValue::Str(key)]) {
                Ok(_) => Flow::Normal,
                Err(err) => Flow::Err(err),
            }
        }
        crate::ast::ReadMode::Line => {
            if let Some(ref prompt_expr) = stmt.prompt {
                match eval_expr(prompt_expr, &state.env, &mut state.fns, context, w, false) {
                    Ok(prompt_val) => {
                        if write!(w, "{}", prompt_val).is_err() || w.flush().is_err() {
                            return Flow::Err(Error::InvalidStatement(None));
                        }
                    }
                    Err(err) => return Flow::Err(err),
                }
            }

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) => Flow::Err(Error::Interpreter(
                    "runtime error: unexpected EOF on stdin".to_string(),
                )),
                Ok(_) => Flow::Normal,
                Err(_) => Flow::Err(Error::Interpreter(
                    "runtime error: failed to read from stdin".to_string(),
                )),
            }
        }
    }
}

pub(super) fn handle_file_write_stmt(
    stmt: &FileWriteStmt,
    state: &mut ProgramState,
    context: &mut RuntimeContext,
    w: &mut dyn Write,
) -> Flow {
    let path_val = match check_eval_result(eval_expr(
        &stmt.path,
        &state.env,
        &mut state.fns,
        context,
        w,
        false,
    )) {
        Ok(val) => val,
        Err(err) => return Flow::Err(err),
    };

    let path_str = match path_val {
        DolangValue::Str(s) => s,
        _ => return Flow::Err(Error::Interpreter("FILE path must be a String".to_string())),
    };

    let mode_str = if let Some(mode_expr) = &stmt.mode {
        match check_eval_result(eval_expr(
            mode_expr,
            &state.env,
            &mut state.fns,
            context,
            w,
            false,
        )) {
            Ok(val) => Some(val.to_string()),
            Err(_) => None,
        }
    } else {
        None
    };

    match mode_str.as_deref() {
        Some("DEL") => {
            match context.call_intrinsic(IntrinsicId::FsDelete, &[DolangValue::Str(path_str)]) {
                Ok(_) => Flow::Normal,
                Err(err) => Flow::Err(err),
            }
        }
        Some("W") | Some("w") | Some("A") | Some("a") | None => {
            if let Some(content_expr) = &stmt.content {
                let content_val = match check_eval_result(eval_expr(
                    content_expr,
                    &state.env,
                    &mut state.fns,
                    context,
                    w,
                    false,
                )) {
                    Ok(val) => val,
                    Err(err) => return Flow::Err(err),
                };

                let content_str = match content_val {
                    DolangValue::Str(s) => s,
                    _ => {
                        return Flow::Err(Error::Interpreter(
                            "FILE content must be a String".to_string(),
                        ));
                    }
                };

                match mode_str.as_deref() {
                    Some("A") | Some("a") | Some("append") => match context.call_intrinsic(
                        IntrinsicId::FsAppendText,
                        &[
                            DolangValue::Str(path_str.clone()),
                            DolangValue::Str(content_str),
                        ],
                    ) {
                        Ok(_) => Flow::Normal,
                        Err(err) => Flow::Err(err),
                    },
                    _ => match context.call_intrinsic(
                        IntrinsicId::FsWriteText,
                        &[
                            DolangValue::Str(path_str.clone()),
                            DolangValue::Str(content_str),
                        ],
                    ) {
                        Ok(_) => Flow::Normal,
                        Err(err) => Flow::Err(err),
                    },
                }
            } else {
                Flow::Return(Some(DolangValue::File {
                    path: path_str,
                    mode: mode_str.clone(),
                }))
            }
        }
        Some(mode) => Flow::Err(Error::Interpreter(format!(
            "invalid file mode '{}', supported modes: \"W\", \"A\", \"DEL\"",
            mode
        ))),
    }
}

pub(super) fn handle_file_read_stmt(
    stmt: &FileReadStmt,
    state: &mut ProgramState,
    context: &mut RuntimeContext,
    w: &mut dyn Write,
) -> Flow {
    let path_val = match check_eval_result(eval_expr(
        &stmt.path,
        &state.env,
        &mut state.fns,
        context,
        w,
        false,
    )) {
        Ok(val) => val,
        Err(err) => return Flow::Err(err),
    };

    let path_str = match path_val {
        DolangValue::Str(s) => s,
        _ => return Flow::Err(Error::Interpreter("FILE path must be a String".to_string())),
    };

    let is_dir =
        match context.call_intrinsic(IntrinsicId::FsIsDir, &[DolangValue::Str(path_str.clone())]) {
            Ok(DolangValue::Bool(is_dir)) => is_dir,
            Ok(_) => false,
            Err(err) => return Flow::Err(err),
        };

    if is_dir {
        return Flow::Err(Error::Interpreter(format!(
            "'{}' is a directory, not a file",
            path_str
        )));
    }

    let exists = match context
        .call_intrinsic(IntrinsicId::FsExists, &[DolangValue::Str(path_str.clone())])
    {
        Ok(DolangValue::Bool(exists)) => exists,
        Ok(_) => false,
        Err(err) => return Flow::Err(err),
    };

    if !exists {
        return Flow::Err(Error::Interpreter(format!("file not found: {}", path_str)));
    }

    let mode_val = if let Some(mode_expr) = &stmt.mode {
        check_eval_result(eval_expr(
            mode_expr,
            &state.env,
            &mut state.fns,
            context,
            w,
            false,
        ))
        .ok()
    } else {
        None
    };

    let read_id = if let Some(mode) = mode_val {
        if mode.to_string() == "LINES" {
            IntrinsicId::FsReadLines
        } else {
            IntrinsicId::FsReadText
        }
    } else {
        IntrinsicId::FsReadText
    };

    let result = match context.call_intrinsic(read_id, &[DolangValue::Str(path_str.clone())]) {
        Ok(value) => value,
        Err(err) => return Flow::Err(err),
    };

    state.env.insert("__FILE_READ_RESULT__".to_string(), result);
    Flow::Normal
}
