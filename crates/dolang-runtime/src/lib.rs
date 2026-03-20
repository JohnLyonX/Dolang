#![allow(clippy::result_large_err)]

pub mod ast {
    pub use dolang_frontend::ast::*;
}

pub mod diagnostics {
    pub use dolang_frontend::diagnostics::*;
}

pub mod error {
    pub use dolang_frontend::error::*;
}

pub mod lexer {
    pub use dolang_frontend::lexer::*;
}

pub mod parser {
    pub use dolang_frontend::parser::*;
}

pub mod syntax {
    pub use dolang_frontend::syntax::*;
}

pub mod token {
    pub use dolang_frontend::token::*;
}

#[path = "../../../src/config.rs"]
pub mod config;
#[path = "../../../src/interpreter/mod.rs"]
pub mod interpreter;
#[path = "../../../src/module/mod.rs"]
pub mod module;
#[path = "../../../src/runtime/mod.rs"]
pub mod runtime;

pub use config::{ProjectConfig, ServerConfig};
pub use interpreter::{DolangValue, FnEnv, exec};
pub use module::{ModuleNamespace, ModuleResolver, ResolvedModule};
pub use parser::parse;
pub use runtime::{
    IntrinsicCall, IntrinsicId, IntrinsicRegistry, ProgramState, RuntimeContext, RuntimeMode,
    RuntimePolicy,
};
