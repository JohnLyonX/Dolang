use std::io::{self, Write};

use crate::ast::Stmt;
use crate::error::Error;
use crate::interpreter::env::{ConstEnv, Env, FnEnv, TypeEnv};
use crate::interpreter::exec::exec_with_writer;
use crate::parser;

use super::RuntimeContext;

#[derive(Debug, Default, Clone)]
pub struct ProgramState {
    pub env: Env,
    pub type_env: TypeEnv,
    pub const_env: ConstEnv,
    pub fns: FnEnv,
}

impl ProgramState {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn execute_program_with_writer(
    statements: &[Stmt],
    state: &mut ProgramState,
    context: &mut RuntimeContext,
    writer: &mut dyn Write,
) -> Result<bool, Error> {
    if let Some(global_cors) = statements.iter().find_map(|stmt| match stmt {
        Stmt::MainDecl(main) => Some(main.global_cors.clone()),
        _ => None,
    }) {
        context.set_global_cors(global_cors);
    }

    for stmt in statements {
        let (should_continue, result) = exec_with_writer(stmt, state, context, writer);
        result?;
        if !should_continue {
            return Ok(false);
        }
    }

    Ok(true)
}

pub fn execute_source_with_writer(
    source: &str,
    state: &mut ProgramState,
    context: &mut RuntimeContext,
    writer: &mut dyn Write,
) -> Result<bool, Error> {
    let statements = parser::parse(source)?;
    execute_program_with_writer(&statements, state, context, writer)
}

#[allow(dead_code)]
pub fn execute_program(
    statements: &[Stmt],
    state: &mut ProgramState,
    context: &mut RuntimeContext,
) -> Result<bool, Error> {
    execute_program_with_writer(statements, state, context, &mut io::stdout())
}
