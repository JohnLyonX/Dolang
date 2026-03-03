// Statement execution - executes AST statements.
use crate::ast::{Expr, FnDeclStmt, Stmt};
use crate::error::Error;
use std::collections::HashMap;
use std::io::{self, Write};

use super::env::{detect_type, format_list, format_map, list_len, list_set, map_set, parse_type_annotation, to_bool, type_name, ValueType, VarValue};
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
                    // Check if this is a function reference and call it
                    if val.starts_with("__anon_fn_") || val.starts_with("__fn_") {
                        // Try to find the user-defined variable name for this function
                        let user_var_name = env.iter()
                            .find(|(_, v)| v.value == val)
                            .map(|(k, _)| k.clone());

                        if let Some(fn_def) = fns.get(&val).cloned() {
                            // Call the function with no arguments
                            match call_fn(&fn_def, &[], fns, w) {
                                Ok(Some(result)) => {
                                    // Format and print the result
                                    let display_val = if result.starts_with("__LST__:") {
                                        format_list(&result)
                                    } else if result.starts_with("__MAP__:") {
                                        format_map(&result)
                                    } else {
                                        result
                                    };
                                    if use_stderr {
                                        eprintln!("{}", display_val);
                                    } else {
                                        if writeln!(w, "{}", display_val).is_err() {
                                            return Flow::Err(Error::InvalidStatement(None));
                                        }
                                    }
                                    return Flow::Normal;
                                }
                                Ok(None) => {
                                    // Function returned nothing, print empty line
                                    if use_stderr {
                                        eprintln!("");
                                    } else {
                                        if writeln!(w).is_err() {
                                            return Flow::Err(Error::InvalidStatement(None));
                                        }
                                    }
                                    return Flow::Normal;
                                }
                                Err(e) => {
                                    // If we found a user variable name, use it in the error message
                                    if let Some(var_name) = user_var_name {
                                        let enhanced_error = match &e {
                                            Error::Interpreter(msg) => {
                                                // Replace the internal function name with user variable name
                                                let new_msg = msg.replace(&val, &var_name);
                                                Error::Interpreter(new_msg)
                                            }
                                            _ => e,
                                        };
                                        return Flow::Err(enhanced_error);
                                    }
                                    return Flow::Err(e);
                                }
                            }
                        }
                    }

                    // Format list and map values for display
                    let display_val = if val.starts_with("__LST__:") {
                        format_list(&val)
                    } else if val.starts_with("__MAP__:") {
                        format_map(&val)
                    } else if val.starts_with("__STR__:") {
                        // Remove __STR__: prefix for display
                        val.trim_start_matches("__STR__:").to_string()
                    } else {
                        val.clone()
                    };
                    if use_stderr {
                        eprintln!("{}", display_val);
                    } else {
                        if writeln!(w, "{}", display_val).is_err() {
                            return Flow::Err(Error::InvalidStatement(None));
                        }
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
                    let key = if let Some(ref prompt_expr) = st.prompt {
                        match eval_expr(prompt_expr, env, fns, w, false) {
                            Some(k) => k,
                            None => {
                                return Flow::Err(Error::Interpreter("ENV requires a key".to_string()));
                            }
                        }
                    } else {
                        return Flow::Err(Error::Interpreter("ENV requires a key: $<<ENV(\"KEY\")".to_string()));
                    };

                    // Remove string prefix if present
                    let key = key.trim_start_matches("__STR__:");

                    // Check for empty key
                    if key.is_empty() {
                        return Flow::Err(Error::Interpreter("ENV requires a key: $<<ENV(\"KEY\")".to_string()));
                    }

                    // Read environment variable
                    match std::env::var(key) {
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
                                let prompt = prompt_val.trim_start_matches("__STR__:");
                                // Print prompt without newline
                                if write!(w, "{}", prompt).is_err() {
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
            // Check if a constant with this name already exists
            if let Some(existing) = env.get(&st.name) {
                if existing.is_const {
                    return Flow::Err(Error::Interpreter(format!(
                        "cannot shadow constant '{}' with a variable",
                        st.name
                    )));
                }
            }

            match check_eval_result(eval_expr(&st.value, env, fns, w, false)) {
                Ok(Some(val)) => {
                    let value_type: ValueType;

                    // Check type annotation if present
                    if let Some(ref type_str) = st.type_annotation {
                        let expected_type = parse_type_annotation(type_str);
                        match expected_type {
                            Some(expected) => {
                                let detected = detect_type(&val);
                                if detected != expected {
                                    return Flow::Err(Error::TypeMismatch(format!(
                                        "type error: declared type '{}' does not match value type '{}'\n  hint: change the annotation or the value",
                                        type_str,
                                        type_name(&detected)
                                    )));
                                }
                                value_type = expected;
                            }
                            None => {
                                return Flow::Err(Error::TypeMismatch(format!(
                                    "type error: unknown type '{}', supported types are: Int, Float, String, Bool",
                                    type_str
                                )));
                            }
                        }
                    } else {
                        // No type annotation: use Dynamic type (can change freely)
                        value_type = ValueType::Dynamic;
                    }

                    env.insert(st.name.clone(), VarValue { value: val, value_type, is_const: false });
                    Flow::Normal
                }
                Ok(None) => Flow::Err(Error::InvalidAssignment(None)),
                Err(e) => Flow::Err(e),
            }
        }

        Stmt::ConstDecl(st) => {
            // Check if constant is already defined
            if let Some(existing) = env.get(&st.name) {
                if existing.is_const {
                    return Flow::Err(Error::Interpreter(format!(
                        "constant '{}' is already defined",
                        st.name
                    )));
                }
            }

            match check_eval_result(eval_expr(&st.value, env, fns, w, false)) {
                Ok(Some(val)) => {
                    let detected_type = detect_type(&val);

                    // Check type annotation if present
                    if let Some(ref type_str) = st.type_annotation {
                        let expected_type = parse_type_annotation(type_str);
                        match expected_type {
                            Some(expected) => {
                                if detected_type != expected {
                                    return Flow::Err(Error::TypeMismatch(format!(
                                        "type error: declared type '{}' does not match value type '{}'\n  hint: change the annotation or the value",
                                        type_str,
                                        type_name(&detected_type)
                                    )));
                                }
                            }
                            None => {
                                return Flow::Err(Error::TypeMismatch(format!(
                                    "type error: unknown type '{}', supported types are: Int, Float, String, Bool",
                                    type_str
                                )));
                            }
                        }
                    }

                    env.insert(st.name.clone(), VarValue { value: val, value_type: detected_type, is_const: true });
                    Flow::Normal
                }
                Ok(None) => Flow::Err(Error::InvalidAssignment(None)),
                Err(e) => Flow::Err(e),
            }
        }

        Stmt::Assign(st) => {
            // Check if this is an index assignment: arr[0] = value OR map["key"] = value
            if let Expr::IndexAccess(idx) = &*st.name {
                // Get the variable name (object)
                let var_name = match eval_expr(&idx.object, env, fns, w, true) {
                    Some(n) => n,
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

                if existing.is_const {
                    return Flow::Err(Error::Interpreter(format!("cannot modify constant '{}'", var_name)));
                }

                // Handle List or Map assignment
                if existing.value_type == ValueType::List {
                    // List assignment: arr[0] = value
                    let index = match idx_val.parse::<usize>() {
                        Ok(i) => i,
                        Err(_) => return Flow::Err(Error::Interpreter("list index must be an integer".to_string())),
                    };

                    let new_list = match list_set(&existing.value, index, val.clone()) {
                        Some(l) => l,
                        None => return Flow::Err(Error::Interpreter(format!(
                            "index out of bounds: list length is {} but index is {}",
                            list_len(&existing.value).unwrap_or(0),
                            index
                        ))),
                    };

                    env.insert(var_name, VarValue { value: new_list, value_type: ValueType::List, is_const: false });
                } else if existing.value_type == ValueType::Map {
                    // Map assignment: map["key"] = value
                    let new_map = match map_set(&existing.value, idx_val, val.clone()) {
                        Some(m) => m,
                        None => return Flow::Err(Error::Interpreter("failed to set map value".to_string())),
                    };

                    env.insert(var_name, VarValue { value: new_map, value_type: ValueType::Map, is_const: false });
                } else {
                    return Flow::Err(Error::Interpreter(format!("cannot index into type {:?}", existing.value_type)));
                }

                return Flow::Normal;
            }

            // Regular assignment: name = value
            let name = match eval_expr(&st.name, env, fns, w, true) {
                Some(n) => n,
                None => return Flow::Err(Error::InvalidAssignment(None)),
            };
            let val = match check_eval_result(eval_expr(&st.value, env, fns, w, false)) {
                Ok(Some(v)) => v,
                Ok(None) => return Flow::Err(Error::InvalidAssignment(None)),
                Err(e) => return Flow::Err(e),
            };

            if let Some(existing) = env.get(&name) {
                if existing.is_const {
                    return Flow::Err(Error::Interpreter(
                        format!("cannot reassign constant '{}'", name),
                    ));
                }
                let new_type = detect_type(&val);
                // Allow assignment if variable is Dynamic (no type annotation) or types match
                if existing.value_type != ValueType::Dynamic && new_type != existing.value_type {
                    return Flow::Err(Error::TypeMismatch(format!(
                        "type error: variable '{}' is declared as '{}', cannot assign '{}' value\n  hint: use '$ {}: {} = ...' to redeclare with a new type",
                        name,
                        type_name(&existing.value_type),
                        type_name(&new_type),
                        name,
                        type_name(&new_type)
                    )));
                }
                // For Dynamic type, keep it as Dynamic to allow future type changes
                let final_type = existing.value_type.clone();
                env.insert(name, VarValue { value: val, value_type: final_type, is_const: false });
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

        Stmt::ForIn(st) => {
            // Evaluate the iterable
            let iterable_val = match eval_expr(&st.iterable, env, fns, w, false) {
                Some(v) => v,
                None => return Flow::Err(Error::InvalidExpression(None)),
            };

            // Determine what to iterate over
            if iterable_val.starts_with("__LST__:") {
                // Iterate over list elements
                let list = super::env::deserialize_list(&iterable_val).unwrap_or_default();
                for item in list {
                    // Bind the loop variable
                    env.insert(st.var.clone(), VarValue {
                        value: item.clone(),
                        value_type: super::env::detect_type(&item),
                        is_const: false,
                    });

                    // Execute body
                    match exec_block(&st.body, env, fns, w) {
                        Flow::Break => break,
                        Flow::Exit => return Flow::Exit,
                        Flow::Err(e) => return Flow::Err(e),
                        Flow::Return(v) => return Flow::Return(v),
                        Flow::Continue | Flow::Normal => {}
                    }
                }
            } else if iterable_val.starts_with("__MAP__:") {
                // Iterate over map keys
                let map = super::env::deserialize_map(&iterable_val).unwrap_or_default();
                for key in map.keys() {
                    // Bind the loop variable (key)
                    env.insert(st.var.clone(), VarValue {
                        value: key.clone(),
                        value_type: super::env::detect_type(key),
                        is_const: false,
                    });

                    // Execute body
                    match exec_block(&st.body, env, fns, w) {
                        Flow::Break => break,
                        Flow::Exit => return Flow::Exit,
                        Flow::Err(e) => return Flow::Err(e),
                        Flow::Return(v) => return Flow::Return(v),
                        Flow::Continue | Flow::Normal => {}
                    }
                }
            } else if !iterable_val.starts_with("__LST__:") && !iterable_val.starts_with("__MAP__:")
                && iterable_val.parse::<f64>().is_err() && iterable_val != "true" && iterable_val != "false" {
                // Iterate over string characters
                for ch in iterable_val.chars() {
                    let ch_str = ch.to_string();
                    // Bind the loop variable
                    env.insert(st.var.clone(), VarValue {
                        value: ch_str.clone(),
                        value_type: super::env::detect_type(&ch_str),
                        is_const: false,
                    });

                    // Execute body
                    match exec_block(&st.body, env, fns, w) {
                        Flow::Break => break,
                        Flow::Exit => return Flow::Exit,
                        Flow::Err(e) => return Flow::Err(e),
                        Flow::Return(v) => return Flow::Return(v),
                        Flow::Continue | Flow::Normal => {}
                    }
                }
            } else {
                return Flow::Err(Error::Interpreter(format!(
                    "cannot iterate over value of type '{}'",
                    if iterable_val.parse::<f64>().is_ok() {
                        "Number"
                    } else if iterable_val == "true" || iterable_val == "false" {
                        "Bool"
                    } else {
                        "Unknown"
                    }
                )));
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
                        _ => Flow::Err(Error::InvalidExpression(None)),
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
