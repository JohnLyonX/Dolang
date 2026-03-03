// Interpreter module - executes AST statements.
pub mod env;
pub mod eval;
pub mod exec;

pub use env::{FnEnv, VarValue, parse_type_annotation, type_name};
pub use exec::exec;
