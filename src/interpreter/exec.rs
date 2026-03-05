// Statement execution - executes AST statements.
use crate::ast::{Expr, FnDeclStmt, Stmt};
use crate::error::Error;
use std::io::{self, Write};

use super::env::{ConstEnv, Env, FnEnv, TypeEnv, ValueType, get_value_type, parse_type_annotation, type_name};
use super::eval::{check_eval_result, eval_expr};
use super::value::DolangValue;

/// Internal control-flow signal returned by exec_inner.
#[derive(Debug)]
pub enum Flow {
    Normal,
    Break,
    Continue,
    Exit,
    Return(Option<DolangValue>),
    Err(Error),
}

/// Exec executes a statement to the default output (stdout).
/// Takes mutable references to type_env and const_env to maintain type and constant information across statements.
pub fn exec(stmt: &Stmt, env: &mut Env, fns: &mut FnEnv, type_env: &mut TypeEnv, const_env: &mut ConstEnv) -> (bool, Result<(), Error>) {
    exec_with_writer(stmt, env, fns, type_env, const_env, &mut io::stdout())
}

/// ExecWithWriter executes a statement and writes prints to `w`.
pub fn exec_with_writer(
    stmt: &Stmt,
    env: &mut Env,
    fns: &mut FnEnv,
    type_env: &mut TypeEnv,
    const_env: &mut ConstEnv,
    w: &mut dyn Write,
) -> (bool, Result<(), Error>) {
    match exec_inner(stmt, env, fns, type_env, const_env, w) {
        Flow::Normal | Flow::Break | Flow::Continue | Flow::Return(_) => (true, Ok(())),
        Flow::Exit => (false, Ok(())),
        Flow::Err(e) => (true, Err(e)),
    }
}

/// Find undefined variable in an expression
fn find_undefined_var(expr: &Expr) -> Option<String> {
    match expr {
        Expr::VarLookup(v) => Some(v.name.as_ref().to_string()),
        Expr::MethodCall(m) => find_undefined_var(&m.object),
        Expr::IndexAccess(i) => find_undefined_var(&i.object),
        _ => None,
    }
}

fn exec_inner(stmt: &Stmt, env: &mut Env, fns: &mut FnEnv, type_env: &mut TypeEnv, const_env: &mut ConstEnv, w: &mut dyn Write) -> Flow {
    match stmt {
        Stmt::Exit(_) => Flow::Exit,
        Stmt::Break(_) => Flow::Break,
        Stmt::Continue(_) => Flow::Continue,

        Stmt::Return(st) => {
            match &st.value {
                Some(expr) => {
                    match check_eval_result(eval_expr(expr, env, fns, w, false)) {
                        Ok(Some(val)) => Flow::Return(Some(val)),
                        Ok(None) => Flow::Err(Error::InvalidExpression(None)),
                        Err(e) => Flow::Err(e),
                    }
                }
                None => Flow::Return(None),
            }
        }

        Stmt::Print(st) => {
            // Determine the target writer (stdout or stderr)
            let use_stderr = st.target == crate::ast::PrintTarget::Stderr;

            match check_eval_result(eval_expr(&st.value, env, fns, w, false)) {
                Ok(Some(val)) => {
                    // Simply use DolangValue's Display implementation
                    if use_stderr {
                        eprintln!("{}", val);
                    } else if writeln!(w, "{}", val).is_err() {
                        return Flow::Err(Error::InvalidStatement(None));
                    }
                    Flow::Normal
                }
                Ok(None) => {
                    // Check for undefined variable - need to handle nested expressions like MethodCall
                    fn find_undefined_var(expr: &Expr) -> Option<String> {
                        match expr {
                            Expr::VarLookup(v) => Some(v.name.as_ref().to_string()),
                            Expr::MethodCall(m) => find_undefined_var(&m.object),
                            Expr::IndexAccess(i) => find_undefined_var(&i.object),
                            _ => None,
                        }
                    }
                    if let Some(var_name) = find_undefined_var(&st.value) {
                        Flow::Err(Error::Interpreter(format!(
                            "variable '{}' is not defined",
                            var_name
                        )))
                    } else {
                        Flow::Err(Error::InvalidExpression(None))
                    }
                }
                Err(e) => Flow::Err(e),
            }
        }

        Stmt::Read(st) => {
            use std::io;

            match st.mode {
                crate::ast::ReadMode::Env => {
                    // ENV mode: get key from prompt (which contains the key name)
                    // We stored the key in prompt as a string literal
                    let key_val = if let Some(ref prompt_expr) = st.prompt {
                        match eval_expr(prompt_expr, env, fns, w, false) {
                            Some(k) => k,
                            None => {
                                return Flow::Err(Error::Interpreter("ENV requires a key".to_string()));
                            }
                        }
                    } else {
                        return Flow::Err(Error::Interpreter("ENV requires a key: $<<ENV(\"KEY\")".to_string()));
                    };

                    // Convert DolangValue to string
                    let key = key_val.to_string();

                    // Check for empty key
                    if key.is_empty() {
                        return Flow::Err(Error::Interpreter("ENV requires a key: $<<ENV(\"KEY\")".to_string()));
                    }

                    // Read environment variable
                    match std::env::var(&key) {
                        Ok(_val) => {
                            // ENV as standalone statement should not print the value
                            // The value is only used when assigned: $ x = $<<ENV("KEY")
                            // Currently we just return Normal without printing
                            Flow::Normal
                        }
                        Err(_) => {
                            Flow::Err(Error::Interpreter(format!(
                                "runtime error: environment variable '{}' is not defined",
                                key
                            )))
                        }
                    }
                }
                crate::ast::ReadMode::Line => {
                    // LINE mode: read a line from stdin

                    // If there's a prompt, print it first (without newline)
                    if let Some(ref prompt_expr) = st.prompt {
                        match eval_expr(prompt_expr, env, fns, w, false) {
                            Some(prompt_val) => {
                                // Print prompt without newline - use DolangValue's Display
                                if write!(w, "{}", prompt_val).is_err() {
                                    return Flow::Err(Error::InvalidStatement(None));
                                }
                                if w.flush().is_err() {
                                    return Flow::Err(Error::InvalidStatement(None));
                                }
                            }
                            None => {
                                return Flow::Err(Error::InvalidExpression(None));
                            }
                        }
                    }

                    // Read from stdin
                    let mut input = String::new();
                    match io::stdin().read_line(&mut input) {
                        Ok(0) => {
                            // EOF reached
                            Flow::Err(Error::Interpreter("runtime error: unexpected EOF on stdin".to_string()))
                        }
                        Ok(_) => {
                            // Remove trailing newline
                            let _input = input.trim_end_matches('\n').trim_end_matches('\r');
                            // For LINE mode in a statement context (not assignment),
                            // we just read and discard the input (pause effect)
                            // The value will be handled by the assignment if present
                            Flow::Normal
                        }
                        Err(_) => {
                            Flow::Err(Error::Interpreter("runtime error: failed to read from stdin".to_string()))
                        }
                    }
                }
            }
        }

        Stmt::VarDecl(st) => {
            // Evaluate the value expression first to check for undefined variables
            let eval_result = eval_expr(&st.value, env, fns, w, false);

            // Check if value is None (undefined variable)
            if eval_result.is_none() {
                // Try to find which variable is undefined
                let undefined_var = find_undefined_var(&st.value);
                return Flow::Err(Error::Interpreter(format!(
                    "variable '{}' is not defined",
                    undefined_var.unwrap_or_else(|| "unknown".to_string())
                )));
            }

            match check_eval_result(eval_result) {
                Ok(Some(val)) => {
                    // Check type annotation if present
                    if let Some(ref type_str) = st.type_annotation {
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
                        // Record the type in type_env for later assignment checking
                        type_env.insert(st.name.clone(), expected_type);
                    }
                    // For variables, we just insert (can overwrite)
                    env.insert(st.name.clone(), val);
                    Flow::Normal
                }
                Ok(None) => Flow::Err(Error::InvalidAssignment(None)),
                Err(e) => Flow::Err(e),
            }
        }

        Stmt::ConstDecl(st) => {
            // Check if a constant/variable with this name already exists
            if env.contains_key(&st.name) {
                return Flow::Err(Error::Interpreter(format!(
                    "constant '{}' is already defined",
                    st.name
                )));
            }

            // Evaluate the value expression first to check for undefined variables
            let eval_result = eval_expr(&st.value, env, fns, w, false);

            // Check if value is None (undefined variable)
            if eval_result.is_none() {
                // Try to find which variable is undefined
                let undefined_var = find_undefined_var(&st.value);
                return Flow::Err(Error::Interpreter(format!(
                    "variable '{}' is not defined",
                    undefined_var.unwrap_or_else(|| "unknown".to_string())
                )));
            }

            match check_eval_result(eval_result) {
                Ok(Some(val)) => {
                    // Check type annotation if present
                    if let Some(ref type_str) = st.type_annotation {
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
                        // Record the type in type_env for later assignment checking
                        type_env.insert(st.name.clone(), expected_type);
                    }
                    // Record in const_env
                    const_env.insert(st.name.clone(), true);
                    env.insert(st.name.clone(), val);
                    Flow::Normal
                }
                Ok(None) => Flow::Err(Error::InvalidAssignment(None)),
                Err(e) => Flow::Err(e),
            }
        }

        Stmt::Assign(st) => {
            // Check if this is an index assignment: arr[0] = value OR map["key"] = value
            if let Expr::IndexAccess(idx) = &*st.name {
                // Get the variable name (object) - should be a string
                let var_name = match eval_expr(&idx.object, env, fns, w, true) {
                    Some(DolangValue::Str(s)) => s.clone(),
                    Some(_) => return Flow::Err(Error::InvalidAssignment(Some("variable name must be a string".to_string()))),
                    None => return Flow::Err(Error::InvalidAssignment(None)),
                };

                // Get the index/key
                let idx_val = match eval_expr(&idx.index, env, fns, w, false) {
                    Some(v) => v,
                    None => return Flow::Err(Error::InvalidAssignment(None)),
                };

                // Get the new value
                let val = match check_eval_result(eval_expr(&st.value, env, fns, w, false)) {
                    Ok(Some(v)) => v,
                    Ok(None) => return Flow::Err(Error::InvalidAssignment(None)),
                    Err(e) => return Flow::Err(e),
                };

                // Get existing value and check type
                let existing = match env.get(&var_name) {
                    Some(e) => e,
                    None => return Flow::Err(Error::Interpreter(format!("variable '{}' not found", var_name))),
                };

                // Handle List or Map assignment using DolangValue
                match existing {
                    DolangValue::List(list) => {
                        // List assignment: arr[0] = value
                        let index = match &idx_val {
                            DolangValue::Int(i) => *i as usize,
                            DolangValue::Float(f) if f.fract() == 0.0 => *f as usize,
                            _ => return Flow::Err(Error::Interpreter("list index must be an integer".to_string())),
                        };

                        if index >= list.len() {
                            return Flow::Err(Error::Interpreter(format!(
                                "index out of bounds: list length is {} but index is {}",
                                list.len(), index
                            )));
                        }

                        let mut new_list = list.clone();
                        new_list[index] = val;
                        env.insert(var_name, DolangValue::List(new_list));
                    }
                    DolangValue::Map(map) => {
                        // Map assignment: map["key"] = value
                        let key = idx_val.to_string();
                        let mut new_map = map.clone();
                        new_map.insert(key, val);
                        env.insert(var_name, DolangValue::Map(new_map));
                    }
                    _ => {
                        return Flow::Err(Error::Interpreter(format!("cannot index into type {}", existing.type_name())));
                    }
                }

                return Flow::Normal;
            }

            // Regular assignment: name = value
            let name = match eval_expr(&st.name, env, fns, w, true) {
                Some(DolangValue::Str(s)) => s.clone(),
                Some(_) => return Flow::Err(Error::InvalidAssignment(Some("variable name must be a string".to_string()))),
                None => return Flow::Err(Error::InvalidAssignment(None)),
            };
            let val = match check_eval_result(eval_expr(&st.value, env, fns, w, false)) {
                Ok(Some(v)) => v,
                Ok(None) => return Flow::Err(Error::InvalidAssignment(None)),
                Err(e) => return Flow::Err(e),
            };

            // Check if variable is a constant
            if let Some(is_const) = const_env.get(&name) {
                if *is_const {
                    return Flow::Err(Error::Interpreter(format!(
                        "cannot reassign constant '{}'",
                        name
                    )));
                }
            }

            // Check type compatibility if variable has declared type
            if let Some(declared_type) = type_env.get(&name) {
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

            // Insert/update the variable
            env.insert(name, val);
            Flow::Normal
        }

        Stmt::If(st) => {
            for branch in &st.branches {
                let cond_val = check_eval_result(eval_expr(&branch.condition, env, fns, w, false));
                match cond_val {
                    Ok(Some(cond_val)) => {
                        if cond_val.is_truthy() {
                            return exec_block(&branch.body, env, fns, type_env, const_env, w);
                        }
                    }
                    Ok(None) => {}
                    Err(e) => return Flow::Err(e),
                }
            }
            exec_block(&st.else_body, env, fns, type_env, const_env, w)
        }

        Stmt::While(st) => {
            loop {
                let cond = check_eval_result(eval_expr(&st.condition, env, fns, w, false));
                match cond {
                    Ok(Some(ref c)) if c.is_truthy() => {}
                    Ok(_) => break,
                    Err(e) => return Flow::Err(e),
                }
                match exec_block(&st.body, env, fns, type_env, const_env, w) {
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
                match exec_block(&st.body, env, fns, type_env, const_env, w) {
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
                let f = exec_inner(init, env, fns, type_env, const_env, w);
                match f {
                    Flow::Normal => {}
                    other => return other,
                }
            }

            loop {
                if let Some(cond_expr) = &st.condition {
                    let cond = check_eval_result(eval_expr(cond_expr, env, fns, w, false));
                    match cond {
                        Ok(Some(ref c)) if c.is_truthy() => {}
                        Ok(_) => break,
                        Err(e) => return Flow::Err(e),
                    }
                }

                match exec_block(&st.body, env, fns, type_env, const_env, w) {
                    Flow::Break => break,
                    Flow::Exit => return Flow::Exit,
                    Flow::Err(e) => return Flow::Err(e),
                    Flow::Return(v) => return Flow::Return(v),
                    Flow::Continue | Flow::Normal => {}
                }

                if let Some(update) = &st.update {
                    let f = exec_inner(update, env, fns, type_env, const_env, w);
                    match f {
                        Flow::Normal => {}
                        other => return other,
                    }
                }
            }
            Flow::Normal
        }

        Stmt::ForIn(st) => {
            // Evaluate the iterable
            let iterable_val = match eval_expr(&st.iterable, env, fns, w, false) {
                Some(v) => v,
                None => return Flow::Err(Error::InvalidExpression(None)),
            };

            // Determine what to iterate over based on DolangValue type
            match iterable_val {
                DolangValue::List(list) => {
                    // Iterate over list elements
                    for item in list {
                        // Bind the loop variable
                        env.insert(st.var.clone(), item.clone());

                        // Execute body
                        match exec_block(&st.body, env, fns, type_env, const_env, w) {
                            Flow::Break => break,
                            Flow::Exit => return Flow::Exit,
                            Flow::Err(e) => return Flow::Err(e),
                            Flow::Return(v) => return Flow::Return(v),
                            Flow::Continue | Flow::Normal => {}
                        }
                    }
                }
                DolangValue::Map(map) => {
                    // Iterate over map keys
                    for key in map.keys() {
                        // Bind the loop variable (key)
                        env.insert(st.var.clone(), DolangValue::Str(key.clone()));

                        // Execute body
                        match exec_block(&st.body, env, fns, type_env, const_env, w) {
                            Flow::Break => break,
                            Flow::Exit => return Flow::Exit,
                            Flow::Err(e) => return Flow::Err(e),
                            Flow::Return(v) => return Flow::Return(v),
                            Flow::Continue | Flow::Normal => {}
                        }
                    }
                }
                DolangValue::Str(s) => {
                    // Iterate over string characters
                    for ch in s.chars() {
                        let ch_str = ch.to_string();
                        // Bind the loop variable
                        env.insert(st.var.clone(), DolangValue::Str(ch_str));

                        // Execute body
                        match exec_block(&st.body, env, fns, type_env, const_env, w) {
                            Flow::Break => break,
                            Flow::Exit => return Flow::Exit,
                            Flow::Err(e) => return Flow::Err(e),
                            Flow::Return(v) => return Flow::Return(v),
                            Flow::Continue | Flow::Normal => {}
                        }
                    }
                }
                _ => {
                    return Flow::Err(Error::Interpreter(format!(
                        "cannot iterate over value of type '{}'",
                        iterable_val.type_name())));
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
                    // Check if result is an error string
                    if let DolangValue::Str(s) = &result
                        && s.starts_with("[FUNC_ERROR]")
                    {
                        let error_msg = s.trim_start_matches("[FUNC_ERROR]");
                        return Flow::Err(Error::Interpreter(error_msg.to_string()));
                    }
                    Flow::Normal
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
                        _ => Flow::Err(Error::InvalidExpression(None)),
                    }
                }
                Err(e) => Flow::Err(e),
            }
        }
    }
}

fn exec_block(stmts: &[Stmt], env: &mut Env, fns: &mut FnEnv, type_env: &mut TypeEnv, const_env: &mut ConstEnv, w: &mut dyn Write) -> Flow {
    for stmt in stmts {
        let f = exec_inner(stmt, env, fns, type_env, const_env, w);
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
    args: &[DolangValue],
    fns: &mut FnEnv,
    w: &mut dyn Write,
) -> Result<Option<DolangValue>, Error> {
    if args.len() != fn_def.params.len() {
        return Err(Error::Interpreter(format!(
            "function '{}' expects {} arguments, got {}",
            fn_def.name,
            fn_def.params.len(),
            args.len()
        )));
    }

    let mut local_env: Env = Env::new();
    let mut local_type_env: TypeEnv = TypeEnv::new();
    let mut local_const_env: ConstEnv = ConstEnv::new();
    for (param, arg) in fn_def.params.iter().zip(args.iter()) {
        local_env.insert(param.clone(), arg.clone());
    }

    let flow = exec_block(&fn_def.body, &mut local_env, fns, &mut local_type_env, &mut local_const_env, w);
    match flow {
        Flow::Return(val) => {
            if let Some(expected_type) = &fn_def.return_type {
                if let Some(ref v) = val {
                    let actual_type = match v {
                        DolangValue::Int(_) => ValueType::Int,
                        DolangValue::Float(_) => ValueType::Float,
                        DolangValue::Str(_) => ValueType::String,
                        DolangValue::Bool(_) => ValueType::Bool,
                        DolangValue::List(_) => ValueType::List,
                        DolangValue::Map(_) => ValueType::Map,
                        DolangValue::Function { .. } => ValueType::Dynamic,
                        DolangValue::Null => ValueType::Dynamic,
                    };
                    let (expected, expected_str) = match expected_type.to_lowercase().as_str() {
                        "int" | "integer" => (ValueType::Int, "Int"),
                        "float" => (ValueType::Float, "Float"),
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
                            ValueType::Dynamic => "Dynamic",
                            ValueType::Int => "Int",
                            ValueType::Float => "Float",
                            ValueType::String => "String",
                            ValueType::Bool => "Bool",
                            ValueType::List => "List",
                            ValueType::Map => "Map",
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
