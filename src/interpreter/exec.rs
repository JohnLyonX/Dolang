// Statement execution - executes AST statements.
use crate::ast::{Expr, FnDeclStmt, Stmt};
use crate::error::Error;
use std::collections::HashMap;
use std::io::{self, Write};

use super::env::{detect_type, to_bool, ValueType, VarValue};
use super::eval::{check_eval_result, eval_expr};

pub type Env = HashMap<String, VarValue>;
pub type FnEnv = HashMap<String, FnDeclStmt>;

/// Internal control-flow signal returned by exec_inner.
#[derive(Debug)]
pub enum Flow {
    Normal,
    Break,
    Continue,
    Exit,
    Return(Option<String>),
    Err(Error),
}

/// Exec executes a statement to the default output (stdout).
pub fn exec(stmt: &Stmt, env: &mut Env, fns: &mut FnEnv) -> (bool, Result<(), Error>) {
    exec_with_writer(stmt, env, fns, &mut io::stdout())
}

/// ExecWithWriter executes a statement and writes prints to `w`.
pub fn exec_with_writer(
    stmt: &Stmt,
    env: &mut Env,
    fns: &mut FnEnv,
    w: &mut dyn Write,
) -> (bool, Result<(), Error>) {
    match exec_inner(stmt, env, fns, w) {
        Flow::Normal | Flow::Break | Flow::Continue | Flow::Return(_) => (true, Ok(())),
        Flow::Exit => (false, Ok(())),
        Flow::Err(e) => (true, Err(e)),
    }
}

fn exec_inner(stmt: &Stmt, env: &mut Env, fns: &mut FnEnv, w: &mut dyn Write) -> Flow {
    match stmt {
        Stmt::Exit(_) => Flow::Exit,
        Stmt::Break(_) => Flow::Break,
        Stmt::Continue(_) => Flow::Continue,

        Stmt::Return(st) => {
            match &st.value {
                Some(expr) => {
                    match check_eval_result(eval_expr(expr, env, fns, w, false)) {
                        Ok(Some(val)) => Flow::Return(Some(val)),
                        Ok(None) => Flow::Err(Error::InvalidExpression),
                        Err(e) => Flow::Err(e),
                    }
                }
                None => Flow::Return(None),
            }
        }

        Stmt::Print(st) => {
            match check_eval_result(eval_expr(&st.value, env, fns, w, false)) {
                Ok(Some(val)) => {
                    if writeln!(w, "{}", val).is_err() {
                        return Flow::Err(Error::InvalidStatement);
                    }
                    Flow::Normal
                }
                Ok(None) => {
                    if let Expr::VarLookup(v) = &*st.value {
                        Flow::Err(Error::Interpreter(format!(
                            "variable '{}' is not defined",
                            v.name
                        )))
                    } else {
                        Flow::Err(Error::InvalidExpression)
                    }
                }
                Err(e) => Flow::Err(e),
            }
        }

        Stmt::VarDecl(st) => {
            match check_eval_result(eval_expr(&st.value, env, fns, w, false)) {
                Ok(Some(val)) => {
                    let value_type = detect_type(&val);
                    env.insert(st.name.clone(), VarValue { value: val, value_type, is_const: false });
                    Flow::Normal
                }
                Ok(None) => Flow::Err(Error::InvalidAssignment),
                Err(e) => Flow::Err(e),
            }
        }

        Stmt::ConstDecl(st) => {
            match check_eval_result(eval_expr(&st.value, env, fns, w, false)) {
                Ok(Some(val)) => {
                    let value_type = detect_type(&val);
                    env.insert(st.name.clone(), VarValue { value: val, value_type, is_const: true });
                    Flow::Normal
                }
                Ok(None) => Flow::Err(Error::InvalidAssignment),
                Err(e) => Flow::Err(e),
            }
        }

        Stmt::Assign(st) => {
            let name = match eval_expr(&st.name, env, fns, w, true) {
                Some(n) => n,
                None => return Flow::Err(Error::InvalidAssignment),
            };
            let val = match check_eval_result(eval_expr(&st.value, env, fns, w, false)) {
                Ok(Some(v)) => v,
                Ok(None) => return Flow::Err(Error::InvalidAssignment),
                Err(e) => return Flow::Err(e),
            };

            if let Some(existing) = env.get(&name) {
                if existing.is_const {
                    return Flow::Err(Error::Interpreter(
                        format!("cannot reassign constant '{}'", name),
                    ));
                }
                let new_type = detect_type(&val);
                if new_type != existing.value_type {
                    return Flow::Err(Error::Interpreter(format!(
                        "type mismatch: cannot assign {} to variable '{}' of type {:?}",
                        val, name, existing.value_type
                    )));
                }
                env.insert(name, VarValue { value: val, value_type: new_type, is_const: false });
            } else {
                let value_type = detect_type(&val);
                env.insert(name, VarValue { value: val, value_type, is_const: false });
            }
            Flow::Normal
        }

        Stmt::If(st) => {
            for branch in &st.branches {
                let cond_val = check_eval_result(eval_expr(&branch.condition, env, fns, w, false));
                match cond_val {
                    Ok(Some(cond_str)) => {
                        if to_bool(&cond_str) {
                            return exec_block(&branch.body, env, fns, w);
                        }
                    }
                    Ok(None) => {}
                    Err(e) => return Flow::Err(e),
                }
            }
            exec_block(&st.else_body, env, fns, w)
        }

        Stmt::While(st) => {
            loop {
                let cond = check_eval_result(eval_expr(&st.condition, env, fns, w, false));
                match cond {
                    Ok(Some(ref c)) if to_bool(c) => {}
                    Ok(_) => break,
                    Err(e) => return Flow::Err(e),
                }
                match exec_block(&st.body, env, fns, w) {
                    Flow::Break => break,
                    Flow::Exit => return Flow::Exit,
                    Flow::Err(e) => return Flow::Err(e),
                    Flow::Return(v) => return Flow::Return(v),
                    Flow::Continue | Flow::Normal => {}
                }
            }
            Flow::Normal
        }

        Stmt::Loop(st) => {
            loop {
                match exec_block(&st.body, env, fns, w) {
                    Flow::Break => break,
                    Flow::Exit => return Flow::Exit,
                    Flow::Err(e) => return Flow::Err(e),
                    Flow::Return(v) => return Flow::Return(v),
                    Flow::Continue | Flow::Normal => {}
                }
            }
            Flow::Normal
        }

        Stmt::For(st) => {
            if let Some(init) = &st.init {
                let f = exec_inner(init, env, fns, w);
                match f {
                    Flow::Normal => {}
                    other => return other,
                }
            }

            loop {
                if let Some(cond_expr) = &st.condition {
                    let cond = check_eval_result(eval_expr(cond_expr, env, fns, w, false));
                    match cond {
                        Ok(Some(ref c)) if to_bool(c) => {}
                        Ok(_) => break,
                        Err(e) => return Flow::Err(e),
                    }
                }

                match exec_block(&st.body, env, fns, w) {
                    Flow::Break => break,
                    Flow::Exit => return Flow::Exit,
                    Flow::Err(e) => return Flow::Err(e),
                    Flow::Return(v) => return Flow::Return(v),
                    Flow::Continue | Flow::Normal => {}
                }

                if let Some(update) = &st.update {
                    let f = exec_inner(update, env, fns, w);
                    match f {
                        Flow::Normal => {}
                        other => return other,
                    }
                }
            }
            Flow::Normal
        }

        Stmt::FnDecl(st) => {
            if fns.contains_key(&st.name) {
                return Flow::Err(Error::Interpreter(format!(
                    "function '{}' is already defined",
                    st.name
                )));
            }
            fns.insert(st.name.clone(), st.clone());
            Flow::Normal
        }

        Stmt::ExprStmt(expr) => {
            let eval_result = check_eval_result(eval_expr(expr, env, fns, w, false));
            match eval_result {
                Ok(Some(result)) => {
                    if result.starts_with("[FUNC_ERROR]") {
                        let error_msg = result.trim_start_matches("[FUNC_ERROR]");
                        Flow::Err(Error::Interpreter(error_msg.to_string()))
                    } else {
                        Flow::Normal
                    }
                }
                Ok(None) => {
                    match &**expr {
                        Expr::FnCall(call) => {
                            if fns.contains_key(&call.name) {
                                Flow::Err(Error::Interpreter(format!(
                                    "error calling function '{}'",
                                    call.name
                                )))
                            } else {
                                Flow::Err(Error::Interpreter(format!(
                                    "function '{}' is not defined",
                                    call.name
                                )))
                            }
                        }
                        Expr::VarLookup(v) => Flow::Err(Error::Interpreter(format!(
                            "variable '{}' is not defined",
                            v.name
                        ))),
                        _ => Flow::Err(Error::InvalidExpression),
                    }
                }
                Err(e) => Flow::Err(e),
            }
        }
    }
}

fn exec_block(stmts: &[Stmt], env: &mut Env, fns: &mut FnEnv, w: &mut dyn Write) -> Flow {
    for stmt in stmts {
        let f = exec_inner(stmt, env, fns, w);
        match f {
            Flow::Normal => {}
            other => return other,
        }
    }
    Flow::Normal
}

/// Call a user-defined function with the given arguments.
pub fn call_fn(
    fn_def: &FnDeclStmt,
    args: &[String],
    fns: &mut FnEnv,
    w: &mut dyn Write,
) -> Result<Option<String>, Error> {
    if args.len() != fn_def.params.len() {
        return Err(Error::Interpreter(format!(
            "function '{}' expects {} arguments, got {}",
            fn_def.name,
            fn_def.params.len(),
            args.len()
        )));
    }

    let mut local_env: Env = HashMap::new();
    for (param, arg) in fn_def.params.iter().zip(args.iter()) {
        let value_type = detect_type(arg);
        local_env.insert(
            param.clone(),
            VarValue {
                value: arg.clone(),
                value_type,
                is_const: false,
            },
        );
    }

    let flow = exec_block(&fn_def.body, &mut local_env, fns, w);
    match flow {
        Flow::Return(val) => {
            if let Some(expected_type) = &fn_def.return_type {
                if let Some(ref v) = val {
                    let actual_type = detect_type(v);
                    let (expected, expected_str) = match expected_type.to_lowercase().as_str() {
                        "int" | "integer" => (ValueType::Number, "Int"),
                        "float" => (ValueType::Number, "Float"),
                        "string" => (ValueType::String, "String"),
                        "bool" | "boolean" => (ValueType::Bool, "Bool"),
                        _ => {
                            return Err(Error::Interpreter(format!(
                                "unknown return type '{}' for function '{}'",
                                expected_type, fn_def.name
                            )));
                        }
                    };
                    if actual_type != expected {
                        let actual_str = match actual_type {
                            ValueType::Number => "Int",
                            ValueType::String => "String",
                            ValueType::Bool => "Bool",
                        };
                        return Err(Error::Interpreter(format!(
                            "function '{}' expects return type '{}' but got '{}'",
                            fn_def.name, expected_str, actual_str
                        )));
                    }
                } else {
                    return Err(Error::Interpreter(format!(
                        "function '{}' expects return type '{}' but returned nothing",
                        fn_def.name, expected_type
                    )));
                }
            }
            Ok(val)
        }
        Flow::Normal => Ok(None),
        Flow::Err(e) => Err(e),
        Flow::Exit => Ok(None),
        Flow::Break | Flow::Continue => {
            Err(Error::Interpreter(
                "break/continue used outside of loop".to_string(),
            ))
        }
    }
}
