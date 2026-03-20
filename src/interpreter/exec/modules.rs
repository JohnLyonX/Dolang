use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::ast::{FnDeclStmt, ModDeclStmt, Span, Spanned, StaticStmt, Stmt};
use crate::diagnostics::{Diagnostic, codes};
use crate::error::Error;
use crate::module::{ModuleResolver, ResolvedModule};
use crate::runtime::{ProgramState, RuntimeContext};

use super::super::StaticRoute;
use super::super::env::FnEnv;
use super::super::value::DolangValue;
use super::Flow;

pub(super) fn handle_mod_decl(
    stmt: &ModDeclStmt,
    state: &mut ProgramState,
    context: &mut RuntimeContext,
) -> Flow {
    let resolver = ModuleResolver::new(context.project_root().to_path_buf())
        .with_current_file(context.current_file().map(Path::new).map(Path::to_path_buf))
        .with_manifest(context.project_config().cloned());

    let modules = if stmt.wildcard {
        match resolve_wildcard_modules(&resolver, &stmt.path) {
            Ok(modules) => modules,
            Err(err) => return Flow::Err(module_error(stmt.span(), err.to_string())),
        }
    } else {
        let Some(resolved) = resolver.resolve_module(&stmt.path) else {
            return Flow::Err(module_error(
                stmt.span(),
                format!(
                    "module not found: '{}' (tried: {})",
                    stmt.path,
                    resolver.describe_search_order(&stmt.path)
                ),
            ));
        };
        vec![resolved]
    };

    for resolved in modules {
        let namespace_name = resolved
            .file_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default()
            .to_string();
        if namespace_name.is_empty() {
            return Flow::Err(module_error(
                stmt.span(),
                format!(
                    "invalid module file name: '{}'",
                    resolved.file_path.display()
                ),
            ));
        }

        let module_value = match build_module_namespace(&resolved) {
            Ok(module_value) => module_value,
            Err(err) => return Flow::Err(module_error(stmt.span(), err.to_string())),
        };

        if let Err(err) = register_module_namespace(state, &namespace_name, module_value, &resolved)
        {
            return Flow::Err(module_error(stmt.span(), err.to_string()));
        }
    }

    Flow::Normal
}

fn module_error(span: Span, message: impl Into<String>) -> Error {
    Error::Diagnostic(Diagnostic::error(codes::RUNTIME_MODULE_LOAD, message).with_span(span))
}

fn resolve_wildcard_modules(
    resolver: &ModuleResolver,
    module_path: &str,
) -> Result<Vec<ResolvedModule>, Error> {
    let Some(directory) = resolver.resolve_module_directory(module_path) else {
        return Err(Error::Interpreter(format!(
            "module directory not found: '{}.*' (tried: {})",
            module_path,
            resolver.describe_directory_search_order(module_path)
        )));
    };

    let mut modules = Vec::new();
    let read_dir = fs::read_dir(&directory.file_path).map_err(|err| {
        Error::Interpreter(format!(
            "cannot read module directory '{}': {}",
            directory.file_path.display(),
            err
        ))
    })?;

    for entry in read_dir {
        let entry = entry.map_err(|err| {
            Error::Interpreter(format!(
                "cannot scan module directory '{}': {}",
                directory.file_path.display(),
                err
            ))
        })?;
        let file_path = entry.path();
        if !file_path.is_file() || file_path.extension().and_then(|ext| ext.to_str()) != Some("dol")
        {
            continue;
        }

        let file_stem = file_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| {
                Error::Interpreter(format!(
                    "invalid module file name: '{}'",
                    file_path.display()
                ))
            })?;

        let resolved_path = if module_path.is_empty() {
            file_stem.to_string()
        } else {
            format!("{module_path}.{file_stem}")
        };

        modules.push(ResolvedModule {
            module_path: resolved_path,
            file_path,
            namespace: directory.namespace.clone(),
        });
    }

    modules.sort_by(|left, right| left.file_path.cmp(&right.file_path));

    if modules.is_empty() {
        return Err(Error::Interpreter(format!(
            "module directory '{}' contains no importable modules",
            directory.file_path.display()
        )));
    }

    Ok(modules)
}

fn build_module_namespace(resolved: &ResolvedModule) -> Result<DolangValue, Error> {
    let content = fs::read_to_string(&resolved.file_path).map_err(|err| {
        Error::Interpreter(format!(
            "cannot read module '{}': {}",
            resolved.module_path, err
        ))
    })?;

    let module_stmts = crate::parser::parse(&content).map_err(|err| {
        Error::Interpreter(format!(
            "parse error in module '{}': {}",
            resolved.module_path, err
        ))
    })?;

    let (exports, fns) = collect_module_functions(&resolved.file_path, module_stmts)?;

    Ok(DolangValue::ModuleProxy {
        path: resolved.module_path.clone(),
        exports,
        fns,
    })
}

fn collect_module_functions(
    module_file: &Path,
    statements: Vec<Stmt>,
) -> Result<(FnEnv, FnEnv), Error> {
    let mut exports = FnEnv::new();
    let mut fns = FnEnv::new();

    for stmt in statements {
        if let Stmt::FnDecl(fn_decl) = stmt {
            insert_function(&mut fns, fn_decl.clone(), module_file)?;
            if fn_decl.is_public {
                insert_function(&mut exports, fn_decl, module_file)?;
            }
        }
    }

    Ok((exports, fns))
}

fn insert_function(
    target: &mut FnEnv,
    fn_decl: FnDeclStmt,
    module_file: &Path,
) -> Result<(), Error> {
    if let Some(existing) = target.get(&fn_decl.name) {
        return Err(Error::Interpreter(format!(
            "module export conflict in '{}': function '{}' already defined (conflicts with '{}')",
            module_file.display(),
            fn_decl.name,
            existing.name
        )));
    }
    target.insert(fn_decl.name.clone(), fn_decl);
    Ok(())
}

fn register_module_namespace(
    state: &mut ProgramState,
    namespace_name: &str,
    module_value: DolangValue,
    resolved: &ResolvedModule,
) -> Result<(), Error> {
    if let Some(existing) = state.env.get(namespace_name) {
        return Err(Error::Interpreter(format!(
            "module namespace conflict: '{}' from '{}' collides with existing value '{}'",
            namespace_name,
            resolved.file_path.display(),
            existing.type_name()
        )));
    }

    if state.fns.contains_key(namespace_name) {
        return Err(Error::Interpreter(format!(
            "module namespace conflict: '{}' from '{}' collides with existing function '{}'",
            namespace_name,
            resolved.file_path.display(),
            namespace_name
        )));
    }

    if state.const_env.contains_key(namespace_name) {
        return Err(Error::Interpreter(format!(
            "module namespace conflict: '{}' from '{}' collides with existing constant '{}'",
            namespace_name,
            resolved.file_path.display(),
            namespace_name
        )));
    }

    state.env.insert(namespace_name.to_string(), module_value);
    Ok(())
}

pub(super) fn handle_static_stmt(
    stmt: &StaticStmt,
    context: &mut RuntimeContext,
    w: &mut dyn Write,
) -> Flow {
    let base_dir = context
        .current_file()
        .and_then(|f| Path::new(f).parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));

    let static_route = StaticRoute {
        url_prefix: stmt.url_prefix.clone(),
        module_path: stmt.module_path.clone(),
        base_dir: base_dir.display().to_string(),
    };

    context.register_static_route(static_route);
    writeln!(
        w,
        "[INFO] Static route registered: {} -> {}",
        stmt.url_prefix, stmt.module_path
    )
    .ok();
    Flow::Normal
}
