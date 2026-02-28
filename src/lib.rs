pub mod ast;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod syntax;
pub mod token;

pub use parser::parse;
pub use interpreter::{exec, FnEnv, VarValue};
