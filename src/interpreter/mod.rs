// Interpreter module - executes AST statements.
use std::collections::HashMap;

pub mod builtins;
pub mod env;
pub mod eval;
pub mod exec;
pub mod value;

pub use env::{FnEnv, parse_type_annotation, type_name};
pub use exec::exec;
pub use value::DolangValue;

/// HTTP route entry for route registry
#[derive(Debug, Clone)]
pub struct HttpRoute {
    pub method: String,           // "GET", "POST", "PUT", "DELETE", "PATCH"
    pub path: String,              // "/user/:id"
    pub name: String,              // function name
    pub params: Vec<String>,       // parameter names
    pub variadic_param: Option<String>,
    pub return_type: Option<String>,
    pub body: Vec<crate::ast::Stmt>,
}

/// Static file serving entry
#[derive(Debug, Clone)]
pub struct StaticRoute {
    pub url_prefix: String,    // URL prefix, e.g., "/css"
    pub module_path: String,   // module path, e.g., "css" or "css.dolang"
    pub base_dir: String,       // base directory relative to main.dol
}

/// Global HTTP route registry
use std::sync::Mutex;
use once_cell::sync::Lazy;

pub static HTTP_ROUTES: Lazy<Mutex<Vec<HttpRoute>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static STATIC_ROUTES: Lazy<Mutex<Vec<StaticRoute>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Track the current file being executed (for $main() scope checking)
pub static CURRENT_FILE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

/// Set the current file being executed
pub fn set_current_file(filename: Option<String>) {
    if let Ok(mut current) = CURRENT_FILE.lock() {
        *current = filename;
    }
}

/// Get the current file being executed
pub fn get_current_file() -> Option<String> {
    if let Ok(current) = CURRENT_FILE.lock() {
        current.clone()
    } else {
        None
    }
}

/// Clear all registered routes (useful for testing)
pub fn clear_routes() {
    if let Ok(mut routes) = HTTP_ROUTES.lock() {
        routes.clear();
    }
}

/// Get all registered routes
pub fn get_routes() -> Vec<HttpRoute> {
    if let Ok(routes) = HTTP_ROUTES.lock() {
        routes.clone()
    } else {
        Vec::new()
    }
}

/// Print all registered routes (for --routertab option)
pub fn print_routes() {
    let routes = get_routes();
    if routes.is_empty() {
        println!("No routes registered.");
    } else {
        println!("Registered Routes:");
        println!("{:<8} {:<30} → {}", "Method", "Path", "Handler");
        println!("{:<8} {:<30} → {}", "------", "----", "--------");
        for route in routes {
            println!("{:<8} {:<30} → {}", route.method, route.path, route.name);
        }
    }
}

/// Execute HTTP handler body and capture return value
/// Returns (should_continue, result_value, error)
pub fn exec_http_handler(
    body: &[crate::ast::Stmt],
    env: &mut HashMap<String, DolangValue>,
    fns: &mut FnEnv,
    type_env: &mut env::TypeEnv,
    const_env: &mut HashMap<String, bool>,
) -> (bool, Option<DolangValue>, Option<String>) {
    use std::io::sink;
    use super::exec::{exec_inner, Flow};

    let mut output = sink();
    let mut result: Option<DolangValue> = None;

    for stmt in body {
        match exec_inner(stmt, env, fns, type_env, const_env, &mut output) {
            Flow::Normal | Flow::Break | Flow::Continue => {}
            Flow::Exit => return (false, None, Some("exit".to_string())),
            Flow::Return(val) => {
                result = val;
                break;
            }
            Flow::Err(e) => {
                return (true, None, Some(e.to_string()));
            }
        }
    }

    (true, result, None)
}
