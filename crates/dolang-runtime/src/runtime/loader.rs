use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::Stmt;
use crate::diagnostics::{Diagnostic, codes};
use crate::error::Error;
use crate::module::{
    ProjectConfig, resolve_main_file as resolve_manifest_main_file,
    resolve_project_root as resolve_manifest_project_root,
};
use crate::parser;

use super::{RuntimeContext, RuntimeMode};

#[derive(Debug, Clone)]
pub struct LoadedProgram {
    pub path: PathBuf,
    pub source: String,
    pub statements: Vec<Stmt>,
}

pub fn resolve_main_file(path: &Path) -> PathBuf {
    let project_root = resolve_project_root(path);
    let manifest = ProjectConfig::load_from_dir(&project_root);
    resolve_manifest_main_file(path, &project_root, manifest.as_ref())
}

pub fn resolve_project_root(path: &Path) -> PathBuf {
    resolve_manifest_project_root(path)
}

pub fn load_program_from_path(path: &Path) -> Result<LoadedProgram, Error> {
    let resolved_path = path.to_path_buf();
    let source = fs::read_to_string(&resolved_path).map_err(|err| {
        Error::from(
            Diagnostic::error(
                codes::PROJECT_LOAD,
                format!("cannot read file '{}': {}", resolved_path.display(), err),
            )
            .with_file(resolved_path.display().to_string()),
        )
    })?;
    let statements =
        parser::parse(&source).map_err(|err| err.with_file(resolved_path.display().to_string()))?;

    Ok(LoadedProgram {
        path: resolved_path,
        source,
        statements,
    })
}

pub fn load_context_and_program(
    mode: RuntimeMode,
    path: &Path,
) -> Result<(RuntimeContext, LoadedProgram), Error> {
    let project_root = resolve_project_root(path);
    let manifest = ProjectConfig::load_from_dir(&project_root);
    let main_file = resolve_manifest_main_file(path, &project_root, manifest.as_ref());
    if !main_file.exists() {
        return Err(Error::from(
            Diagnostic::error(
                codes::PROJECT_LOAD,
                format!("main file not found: {}", main_file.display()),
            )
            .with_file(main_file.display().to_string()),
        ));
    }

    let mut context = RuntimeContext::new(mode, project_root.clone());
    crate::stdlib_native::register_stdlib_native_modules(&mut context);
    context.set_project_config(manifest);
    context.set_current_file(Some(main_file.to_string_lossy().to_string()));

    let program = load_program_from_path(&main_file)?;
    let global_cors = program.statements.iter().find_map(|stmt| match stmt {
        Stmt::MainDecl(main) => main.global_cors.clone(),
        _ => None,
    });
    context.set_global_cors(global_cors);
    Ok((context, program))
}
