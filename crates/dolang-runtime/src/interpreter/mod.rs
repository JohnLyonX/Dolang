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
    pub method: String,      // "GET", "POST", "PUT", "DELETE", "PATCH"
    pub path: String,        // "/user/:id"
    pub name: String,        // function name
    pub params: Vec<String>, // parameter names
    pub variadic_param: Option<String>,
    pub return_type: Option<String>,
    pub body: Vec<crate::ast::Stmt>,
}

/// Static file serving entry
#[derive(Debug, Clone)]
pub struct StaticRoute {
    pub url_prefix: String,  // URL prefix, e.g., "/css"
    pub module_path: String, // module path, e.g., "css" or "css.dolang"
    pub base_dir: String,    // base directory relative to main.dol
}

/// Execute HTTP handler body and capture return value
/// Returns (should_continue, result_value, error)
pub fn exec_http_handler(
    body: &[crate::ast::Stmt],
    state: &mut crate::runtime::ProgramState,
    context: &mut crate::runtime::RuntimeContext,
) -> (bool, Option<DolangValue>, Option<String>) {
    use super::exec::{Flow, exec_inner};
    use std::io::stdout;

    let mut output = stdout();
    let mut result: Option<DolangValue> = None;

    for stmt in body {
        match exec_inner(stmt, state, context, &mut output) {
            Flow::Normal | Flow::Break | Flow::Continue => {}
            Flow::Exit => return (false, None, Some("exit".to_string())),
            Flow::Return(val) => {
                result = val;
                break;
            }
            Flow::Err(e) => {
                eprintln!("[ERROR] {}", e);
                return (true, None, Some(e.to_string()));
            }
            Flow::Throw(val) => {
                eprintln!("[ERROR] uncaught throw: {val}");
                return (true, None, Some(format!("uncaught throw: {val}")));
            }
        }
    }

    (true, result, None)
}
