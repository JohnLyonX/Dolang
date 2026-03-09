// Statement execution - executes AST statements.
use crate::ast::{Expr, FnDeclStmt, Stmt};
use crate::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::env::{
    ConstEnv, Env, FnEnv, TypeEnv, ValueType, get_value_type, parse_type_annotation, type_name,
};
use super::eval::{check_eval_result, eval_expr};
use super::value::DolangValue;
use super::{HttpRoute, STATIC_ROUTES, StaticRoute, get_current_file};

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
pub fn exec(
    stmt: &Stmt,
    env: &mut Env,
    fns: &mut FnEnv,
    type_env: &mut TypeEnv,
    const_env: &mut ConstEnv,
) -> (bool, Result<(), Error>) {
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

pub(super) fn exec_inner(
    stmt: &Stmt,
    env: &mut Env,
    fns: &mut FnEnv,
    type_env: &mut TypeEnv,
    const_env: &mut ConstEnv,
    w: &mut dyn Write,
) -> Flow {
    match stmt {
        Stmt::Exit(_) => Flow::Exit,
        Stmt::Break(_) => Flow::Break,
        Stmt::Continue(_) => Flow::Continue,

        // --- $mod module declaration ---
        Stmt::ModDecl(stmt) => {
            // Convert module path: "dao.user" -> "dao/user.dol"
            let module_path = stmt.path.replace('.', "/");
            let module_file = format!("{}.dol", module_path);

            // Try to load from current directory first, then from module search path
            let content = if Path::new(&module_file).exists() {
                match fs::read_to_string(&module_file) {
                    Ok(c) => c,
                    Err(e) => {
                        return Flow::Err(Error::Interpreter(format!(
                            "cannot read module '{}': {}",
                            stmt.path, e
                        )));
                    }
                }
            } else {
                // Try with "modules/" prefix
                let module_with_prefix = format!("modules/{}.dol", module_path);
                if Path::new(&module_with_prefix).exists() {
                    match fs::read_to_string(&module_with_prefix) {
                        Ok(c) => c,
                        Err(e) => {
                            return Flow::Err(Error::Interpreter(format!(
                                "cannot read module '{}': {}",
                                stmt.path, e
                            )));
                        }
                    }
                } else {
                    return Flow::Err(Error::Interpreter(format!(
                        "module not found: '{}' (tried: {}, modules/{}.dol)",
                        stmt.path, module_file, module_path
                    )));
                }
            };

            // Parse the module
            let module_stmts = match crate::parser::parse(&content) {
                Ok(stmts) => stmts,
                Err(e) => {
                    return Flow::Err(Error::Interpreter(format!(
                        "parse error in module '{}': {}",
                        stmt.path, e
                    )));
                }
            };

            // Register functions from the module
            for module_stmt in module_stmts {
                if let Stmt::FnDecl(fn_decl) = module_stmt {
                    // Check if function already exists
                    if fns.contains_key(&fn_decl.name) {
                        // Function already exists, skip or could warn
                        continue;
                    }
                    fns.insert(fn_decl.name.clone(), fn_decl);
                }
            }

            Flow::Normal
        }

        // --- $main main entry point ---
        Stmt::MainDecl(stmt) => {
            // Check if $main() is only used in main.dol (allow path prefix like "./main.dol")
            let current_file = get_current_file();
            let is_main_dol = current_file
                .as_ref()
                .map(|f| {
                    std::path::Path::new(f)
                        .file_name()
                        .map(|n| n.to_string_lossy() == "main.dol")
                        .unwrap_or(false)
                })
                .unwrap_or(false);

            if !is_main_dol {
                return Flow::Err(Error::Interpreter(
                    "$main() can only be declared in main.dol".to_string(),
                ));
            }

            // Execute the main function body immediately
            for main_stmt in &stmt.body {
                let (cont, err) = exec(main_stmt, env, fns, type_env, const_env);
                if let Err(e) = err {
                    return Flow::Err(e);
                }
                if !cont {
                    return Flow::Normal;
                }
            }
            Flow::Normal
        }

        Stmt::Return(st) => match &st.value {
            Some(expr) => match check_eval_result(eval_expr(expr, env, fns, w, false)) {
                Ok(Some(val)) => Flow::Return(Some(val)),
                Ok(None) => Flow::Err(Error::InvalidExpression(None)),
                Err(e) => Flow::Err(e),
            },
            None => Flow::Return(None),
        },

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
                                return Flow::Err(Error::Interpreter(
                                    "ENV requires a key".to_string(),
                                ));
                            }
                        }
                    } else {
                        return Flow::Err(Error::Interpreter(
                            "ENV requires a key: $<<ENV(\"KEY\")".to_string(),
                        ));
                    };

                    // Convert DolangValue to string
                    let key = key_val.to_string();

                    // Check for empty key
                    if key.is_empty() {
                        return Flow::Err(Error::Interpreter(
                            "ENV requires a key: $<<ENV(\"KEY\")".to_string(),
                        ));
                    }

                    // Read environment variable
                    match std::env::var(&key) {
                        Ok(_val) => {
                            // ENV as standalone statement should not print the value
                            // The value is only used when assigned: $ x = $<<ENV("KEY")
                            // Currently we just return Normal without printing
                            Flow::Normal
                        }
                        Err(_) => Flow::Err(Error::Interpreter(format!(
                            "runtime error: environment variable '{}' is not defined",
                            key
                        ))),
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
                            Flow::Err(Error::Interpreter(
                                "runtime error: unexpected EOF on stdin".to_string(),
                            ))
                        }
                        Ok(_) => {
                            // Remove trailing newline
                            let _input = input.trim_end_matches('\n').trim_end_matches('\r');
                            // For LINE mode in a statement context (not assignment),
                            // we just read and discard the input (pause effect)
                            // The value will be handled by the assignment if present
                            Flow::Normal
                        }
                        Err(_) => Flow::Err(Error::Interpreter(
                            "runtime error: failed to read from stdin".to_string(),
                        )),
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
                    Some(_) => {
                        return Flow::Err(Error::InvalidAssignment(Some(
                            "variable name must be a string".to_string(),
                        )));
                    }
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
                    None => {
                        return Flow::Err(Error::Interpreter(format!(
                            "variable '{}' not found",
                            var_name
                        )));
                    }
                };

                // Handle List or Map assignment using DolangValue
                match existing {
                    DolangValue::List(list) => {
                        // List assignment: arr[0] = value
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
                        return Flow::Err(Error::Interpreter(format!(
                            "cannot index into type {}",
                            existing.type_name()
                        )));
                    }
                }

                return Flow::Normal;
            }

            // Regular assignment: name = value
            let name = match eval_expr(&st.name, env, fns, w, true) {
                Some(DolangValue::Str(s)) => s.clone(),
                Some(_) => {
                    return Flow::Err(Error::InvalidAssignment(Some(
                        "variable name must be a string".to_string(),
                    )));
                }
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
                        iterable_val.type_name()
                    )));
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

        // --- HTTP function: $GET, $POST, etc. ---
        Stmt::HttpFn(st) => {
            // Register HTTP route in global registry
            use crate::interpreter::{HTTP_ROUTES, HttpRoute};

            let route = HttpRoute {
                method: st.method.clone(),
                path: st.path.clone(),
                name: st.name.clone(),
                params: st.params.clone(),
                variadic_param: st.variadic_param.clone(),
                return_type: st.return_type.clone(),
                body: st.body.clone(),
            };

            if let Ok(mut routes) = HTTP_ROUTES.lock() {
                routes.push(route);
            }

            writeln!(w, "[INFO] HTTP route registered: {} {}", st.method, st.path).ok();
            Flow::Normal
        }

        // --- HTTP block: $HTTP { ... } or $HTTP(path).link(module) ---
        Stmt::HttpBlock(st) => {
            use crate::interpreter::{HTTP_ROUTES, HttpRoute};

            // Handle $HTTP(path).link(module) syntax
            if let Some(link_module) = &st.link {
                // Load the linked module and register its routes with prefix
                let prefix = st.prefix.as_deref().unwrap_or("");
                let module_routes = match load_module_routes(link_module) {
                    Ok(routes) => routes,
                    Err(e) => return Flow::Err(e),
                };

                let route_count = module_routes.len();
                for route in module_routes {
                    // Prepend prefix to the path, handling trailing/leading slashes
                    let prefix_trimmed = prefix.trim_start_matches('/').trim_end_matches('/');
                    let path_trimmed = route.path.trim_start_matches('/');
                    let full_path = if prefix_trimmed.is_empty() {
                        format!("/{}", path_trimmed)
                    } else {
                        format!("/{}/{}", prefix_trimmed, path_trimmed)
                    };
                    let route = HttpRoute {
                        method: route.method.clone(),
                        path: full_path,
                        name: route.name.clone(),
                        params: route.params.clone(),
                        variadic_param: route.variadic_param.clone(),
                        return_type: route.return_type.clone(),
                        body: route.body.clone(),
                    };

                    if let Ok(mut routes) = HTTP_ROUTES.lock() {
                        routes.push(route);
                    }
                }

                writeln!(
                    w,
                    "[INFO] HTTP linked module '{}' with prefix '{}' ({} routes)",
                    link_module, prefix, route_count
                )
                .ok();
                Flow::Normal
            } else {
                // Register all routes in the block ($HTTP { ... } or $HTTP(path) { ... } syntax)
                for route_stmt in &st.routes {
                    // Apply prefix if present
                    let full_path = if let Some(prefix) = &st.prefix {
                        let prefix_trimmed = prefix.trim_start_matches('/').trim_end_matches('/');
                        let path_trimmed = route_stmt.path.trim_start_matches('/');
                        if prefix_trimmed.is_empty() {
                            format!("/{}", path_trimmed)
                        } else {
                            format!("/{}/{}", prefix_trimmed, path_trimmed)
                        }
                    } else {
                        route_stmt.path.clone()
                    };

                    let route = HttpRoute {
                        method: route_stmt.method.clone(),
                        path: full_path,
                        name: route_stmt.name.clone(),
                        params: route_stmt.params.clone(),
                        variadic_param: route_stmt.variadic_param.clone(),
                        return_type: route_stmt.return_type.clone(),
                        body: route_stmt.body.clone(),
                    };

                    if let Ok(mut routes) = HTTP_ROUTES.lock() {
                        routes.push(route);
                    }
                }

                writeln!(w, "[INFO] HTTP block registered {} routes", st.routes.len()).ok();
                Flow::Normal
            }
        }

        // --- Static file serving: $STATIC("/url-prefix", "dir.module") or $STATIC("dir") ---
        Stmt::Static(st) => {
            // Get base directory from current file
            let base_dir = get_current_file()
                .and_then(|f| Path::new(&f).parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| Path::new(".").to_path_buf());

            let static_route = StaticRoute {
                url_prefix: st.url_prefix.clone(),
                module_path: st.module_path.clone(),
                base_dir: base_dir.display().to_string(),
            };

            if let Ok(mut routes) = STATIC_ROUTES.lock() {
                routes.push(static_route);
            }

            writeln!(
                w,
                "[INFO] Static route registered: {} -> {}",
                st.url_prefix, st.module_path
            )
            .ok();
            Flow::Normal
        }

        // --- $>>FILE file write ---
        Stmt::FileWrite(st) => {
            use std::fs::{self, OpenOptions};
            use std::io::Write;

            // Evaluate path
            let path_val = match check_eval_result(eval_expr(&st.path, env, fns, w, false)) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    return Flow::Err(Error::Interpreter("FILE path is required".to_string()));
                }
                Err(e) => return Flow::Err(e),
            };

            let path_str = match path_val {
                DolangValue::Str(s) => s,
                _ => {
                    return Flow::Err(Error::Interpreter("FILE path must be a String".to_string()));
                }
            };

            // Evaluate mode (optional)
            let mode_str = if let Some(mode_expr) = &st.mode {
                match check_eval_result(eval_expr(mode_expr, env, fns, w, false)) {
                    Ok(Some(v)) => Some(v.to_string()),
                    _ => None,
                }
            } else {
                None
            };

            // Handle based on mode
            match mode_str.as_deref() {
                Some("DEL") => {
                    // Delete file
                    match fs::remove_file(&path_str) {
                        Ok(_) => Flow::Normal,
                        Err(e) => {
                            Flow::Err(Error::Interpreter(format!("cannot delete file: {}", e)))
                        }
                    }
                }
                Some("W") | Some("w") | Some("A") | Some("a") | None => {
                    // Check if content is provided directly
                    if let Some(content_expr) = &st.content {
                        let content_val =
                            match check_eval_result(eval_expr(content_expr, env, fns, w, false)) {
                                Ok(Some(v)) => v,
                                Ok(None) => {
                                    return Flow::Err(Error::Interpreter(
                                        "FILE content is required".to_string(),
                                    ));
                                }
                                Err(e) => return Flow::Err(e),
                            };

                        let content_str = match content_val {
                            DolangValue::Str(s) => s,
                            _ => {
                                return Flow::Err(Error::Interpreter(
                                    "FILE content must be a String".to_string(),
                                ));
                            }
                        };

                        // Write directly based on mode
                        match mode_str.as_deref() {
                            Some("A") | Some("a") | Some("append") => {
                                // Append mode
                                match OpenOptions::new().create(true).append(true).open(&path_str) {
                                    Ok(mut file) => match file.write_all(content_str.as_bytes()) {
                                        Ok(_) => Flow::Normal,
                                        Err(e) => Flow::Err(Error::Interpreter(format!(
                                            "write error: {}",
                                            e
                                        ))),
                                    },
                                    Err(e) => Flow::Err(Error::Interpreter(format!(
                                        "cannot open file: {}",
                                        e
                                    ))),
                                }
                            }
                            _ => {
                                // Write mode (default)
                                match fs::write(&path_str, &content_str) {
                                    Ok(_) => Flow::Normal,
                                    Err(e) => {
                                        if e.kind() == std::io::ErrorKind::NotFound {
                                            Flow::Err(Error::Interpreter(format!(
                                                "directory not found for path: {}",
                                                path_str
                                            )))
                                        } else {
                                            Flow::Err(Error::Interpreter(format!(
                                                "write error: {}",
                                                e
                                            )))
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // No content provided - return File object for chaining .content()
                        let file_mode = mode_str.clone();
                        Flow::Return(Some(DolangValue::File {
                            path: path_str,
                            mode: file_mode,
                        }))
                    }
                }
                Some(m) => Flow::Err(Error::Interpreter(format!(
                    "invalid file mode '{}', supported modes: \"W\", \"A\", \"DEL\"",
                    m
                ))),
            }
        }

        // --- $<<FILE file read ---
        Stmt::FileRead(st) => {
            use std::fs;
            use std::io::{self};

            // Evaluate path
            let path_val = match check_eval_result(eval_expr(&st.path, env, fns, w, false)) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    return Flow::Err(Error::Interpreter("FILE path is required".to_string()));
                }
                Err(e) => return Flow::Err(e),
            };

            let path_str = match path_val {
                DolangValue::Str(s) => s,
                _ => {
                    return Flow::Err(Error::Interpreter("FILE path must be a String".to_string()));
                }
            };

            // Check if path is a directory
            let metadata = match fs::metadata(&path_str) {
                Ok(m) => m,
                Err(e) => {
                    if e.kind() == io::ErrorKind::NotFound {
                        return Flow::Err(Error::Interpreter(format!(
                            "file not found: {}",
                            path_str
                        )));
                    }
                    return Flow::Err(Error::Interpreter(format!("cannot access file: {}", e)));
                }
            };

            if metadata.is_dir() {
                return Flow::Err(Error::Interpreter(format!(
                    "'{}' is a directory, not a file",
                    path_str
                )));
            }

            // Evaluate mode (optional)
            let mode_val = if let Some(mode_expr) = &st.mode {
                match check_eval_result(eval_expr(mode_expr, env, fns, w, false)) {
                    Ok(Some(v)) => Some(v),
                    _ => None,
                }
            } else {
                None
            };

            // Handle based on mode
            let result = if let Some(mode) = mode_val {
                let mode_str = mode.to_string();
                if mode_str == "LINES" {
                    // Read all lines
                    match fs::read_to_string(&path_str) {
                        Ok(content) => {
                            let lines: Vec<DolangValue> = content
                                .lines()
                                .map(|s| DolangValue::Str(s.to_string()))
                                .collect();
                            DolangValue::List(lines)
                        }
                        Err(e) => {
                            return Flow::Err(Error::Interpreter(format!(
                                "cannot read file: {}",
                                e
                            )));
                        }
                    }
                } else {
                    // Treat as buffer size - just read all
                    match fs::read_to_string(&path_str) {
                        Ok(content) => DolangValue::Str(content),
                        Err(e) => {
                            return Flow::Err(Error::Interpreter(format!(
                                "cannot read file: {}",
                                e
                            )));
                        }
                    }
                }
            } else {
                // Default: read all content
                match fs::read_to_string(&path_str) {
                    Ok(content) => DolangValue::Str(content),
                    Err(e) => {
                        return Flow::Err(Error::Interpreter(format!("cannot read file: {}", e)));
                    }
                }
            };

            // Store result in a special variable or return it
            // For now, just return it - it will be used if assigned
            env.insert("__FILE_READ_RESULT__".to_string(), result);
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
                Ok(None) => match &**expr {
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
                },
                Err(e) => Flow::Err(e),
            }
        }
    }
}

fn exec_block(
    stmts: &[Stmt],
    env: &mut Env,
    fns: &mut FnEnv,
    type_env: &mut TypeEnv,
    const_env: &mut ConstEnv,
    w: &mut dyn Write,
) -> Flow {
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
    let min_params = fn_def.params.len();
    // Allow variable arguments if there's a variadic param
    let has_variadic = fn_def.variadic_param.is_some();
    if has_variadic && args.len() < min_params {
        return Err(Error::Interpreter(format!(
            "function '{}' expects at least {} arguments, got {}",
            fn_def.name,
            min_params,
            args.len()
        )));
    }
    if !has_variadic && args.len() != min_params {
        return Err(Error::Interpreter(format!(
            "function '{}' expects {} arguments, got {}",
            fn_def.name,
            min_params,
            args.len()
        )));
    }

    let mut local_env: Env = Env::new();
    let mut local_type_env: TypeEnv = TypeEnv::new();
    let mut local_const_env: ConstEnv = ConstEnv::new();

    // Bind regular parameters
    for (param, arg) in fn_def.params.iter().zip(args.iter()) {
        local_env.insert(param.clone(), arg.clone());
    }

    // Bind variadic parameter to a List of remaining args
    if let Some(var_param) = &fn_def.variadic_param {
        let extra_args: Vec<DolangValue> = args[min_params..].to_vec();
        local_env.insert(var_param.clone(), DolangValue::List(extra_args));
    }

    let flow = exec_block(
        &fn_def.body,
        &mut local_env,
        fns,
        &mut local_type_env,
        &mut local_const_env,
        w,
    );
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
                        DolangValue::File { .. } => ValueType::File,
                        DolangValue::Json(_) => ValueType::Json,
                        DolangValue::Html(_) => ValueType::Dynamic,
                        DolangValue::Response { .. } => ValueType::Response,
                        DolangValue::ModuleProxy { .. } => ValueType::Dynamic,
                        DolangValue::Null => ValueType::Dynamic,
                    };
                    let (expected, expected_str) = match expected_type.to_lowercase().as_str() {
                        "int" | "integer" => (ValueType::Int, "Int"),
                        "float" => (ValueType::Float, "Float"),
                        "string" => (ValueType::String, "String"),
                        "bool" | "boolean" => (ValueType::Bool, "Bool"),
                        "json" => (ValueType::Json, "Json"),
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
                            ValueType::File => "File",
                            ValueType::Json => "Json",
                            ValueType::Response => "Response",
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
        Flow::Break | Flow::Continue => Err(Error::Interpreter(
            "break/continue used outside of loop".to_string(),
        )),
    }
}

/// Load HTTP routes from a module file (e.g., "routers.api")
fn load_module_routes(module_path: &str) -> Result<Vec<HttpRoute>, Error> {
    use crate::ast::Stmt;
    use crate::parser;

    // Convert module path to file path
    // "routers.api" -> "routers/api.dol"
    let file_path = module_path.replace('.', "/") + ".dol";

    // Try to find the module file
    // Check in current directory first, then relative to main file
    let current_file = get_current_file();

    // Get current working directory
    let cwd = std::env::current_dir().unwrap_or_default();

    // Search paths: current dir, cwd + file_path, modules/ prefix, and relative to main file
    let mut search_paths = vec![
        Path::new(&file_path).to_path_buf(),
        cwd.join(&file_path),
        cwd.join("modules").join(&file_path),
    ];

    // If main_file has a directory component, search relative to it
    if let Some(main_file) = current_file {
        let main_path = Path::new(&main_file);
        if let Some(main_dir) = main_path.parent() {
            if !main_dir.as_os_str().is_empty() {
                search_paths.push(main_dir.join(&file_path));
                search_paths.push(main_dir.join("modules").join(&file_path));
            }
        }
    }

    let mut full_path = None;
    for p in &search_paths {
        if p.exists() {
            full_path = Some(p.clone());
            break;
        }
    }

    let path = match full_path {
        Some(p) => p,
        None => {
            return Err(Error::Interpreter(format!(
                "module file not found: {}.dol (tried: {:?})",
                module_path, search_paths
            )));
        }
    };

    // Read and parse the module file
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            return Err(Error::Interpreter(format!(
                "cannot read module file '{}': {}",
                path.display(),
                e
            )));
        }
    };

    let statements = match parser::parse(&content) {
        Ok(stmts) => stmts,
        Err(e) => {
            return Err(Error::Interpreter(format!(
                "failed to parse module '{}': {}",
                module_path, e
            )));
        }
    };

    // Extract HTTP routes from the module
    let mut routes = Vec::new();
    for stmt in statements {
        if let Stmt::HttpFn(http_fn) = stmt {
            routes.push(HttpRoute {
                method: http_fn.method,
                path: http_fn.path,
                name: http_fn.name,
                params: http_fn.params,
                variadic_param: http_fn.variadic_param,
                return_type: http_fn.return_type,
                body: http_fn.body,
            });
        }
    }

    Ok(routes)
}
