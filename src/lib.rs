pub mod ast;
pub mod config;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod syntax;
pub mod token;

pub use config::{ProjectConfig, ServerConfig};
pub use parser::parse;
pub use interpreter::{exec, FnEnv, DolangValue};
