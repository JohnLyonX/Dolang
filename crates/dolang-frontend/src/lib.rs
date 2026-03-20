#![allow(clippy::module_inception, clippy::result_large_err)]

#[path = "../../../src/ast/mod.rs"]
pub mod ast;
#[path = "../../../src/diagnostics/mod.rs"]
pub mod diagnostics;
#[path = "../../../src/error/mod.rs"]
pub mod error;
#[path = "../../../src/lexer/mod.rs"]
pub mod lexer;
#[path = "../../../src/parser/mod.rs"]
pub mod parser;
#[path = "../../../src/syntax/mod.rs"]
pub mod syntax;
#[path = "../../../src/token/mod.rs"]
pub mod token;

pub use parser::parse;
