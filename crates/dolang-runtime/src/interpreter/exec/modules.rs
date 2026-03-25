use std::borrow::ToOwned;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::ast::{ModDeclStmt, Span, Spanned, StaticStmt};
use crate::diagnostics::{Diagnostic, codes};
use crate::error::Error;
use crate::module::{ModuleResolver, ResolvedModule};
use crate::runtime::{ProgramState, RuntimeContext, execute_program_with_writer};

use super::super::StaticRoute;
use super::super::env::{Env, FnEnv};
use super::super::value::DolangValue;
use super::Flow;

pub(super) fn handle_mod_decl(
    stmt: &ModDeclStmt,
    state: &mut ProgramState,
    context: &mut RuntimeContext,
    w: &mut dyn Write,
) -> Flow {
    let resolver = ModuleResolver::new(context.project_root().to_path_buf())
        .with_current_file(context.current_file().map(Path::new).map(Path::to_path_buf))
        .with_manifest(context.project_config().cloned());

    let modules = if stmt.wildcard {
        match resolve_wildcard_modules(&resolver, &stmt.path) {
            Ok(modules) => modules,
            Err(err) => return Flow::Err(module_error(stmt.span(), err.to_string())),
        }
    } else if let Some(resolved) = resolver.resolve_module(&stmt.path) {
        vec![resolved]
    } else if let Some(native_exports) = context.native_module(&stmt.path) {
        // Native module: build ModuleProxy from the registry and register directly.
        let namespace = stmt
            .path
            .rsplit('.')
            .next()
            .unwrap_or(&stmt.path)
            .to_string();
        if state.env.contains_key(&namespace) {
            return Flow::Err(module_error(
                stmt.span(),
                format!("module namespace conflict: '{}' already exists", namespace),
            ));
        }
        let module_value = DolangValue::ModuleProxy {
            path: stmt.path.clone(),
            exports: FnEnv::new(),
            fns: FnEnv::new(),
            native_exports: native_exports.clone(),
            module_env: Env::new(),
        };
        state.env.insert(namespace, module_value);
        return Flow::Normal;
    } else {
        return Flow::Err(module_error(
            stmt.span(),
            format!(
                "module not found: '{}' (tried: {})",
                stmt.path,
                resolver.describe_search_order(&stmt.path)
            ),
        ));
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

        let module_value = match build_module_namespace(&resolved, context, w) {
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

fn build_module_namespace(
    resolved: &ResolvedModule,
    context: &mut RuntimeContext,
    w: &mut dyn Write,
) -> Result<DolangValue, Error> {
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

    // 保存并切换 current_file，使模块内的相对 $mod 解析正确
    let prev_file = context.current_file().map(ToOwned::to_owned);
    context.set_current_file(Some(resolved.file_path.to_string_lossy().to_string()));

    // 完整执行模块文件，捕获执行后的状态
    let mut module_state = ProgramState::new();
    let exec_result = execute_program_with_writer(&module_stmts, &mut module_state, context, w);

    // 恢复 current_file
    context.set_current_file(prev_file);

    exec_result.map_err(|err| {
        Error::Interpreter(format!(
            "error in module '{}': {}",
            resolved.module_path, err
        ))
    })?;

    // 从执行后的 fns 提取导出（is_public == true 的函数）
    let fns = module_state.fns.clone();
    let mut exports = FnEnv::new();
    for (name, fn_decl) in &fns {
        if fn_decl.is_public {
            exports.insert(name.clone(), fn_decl.clone());
        }
    }

    Ok(DolangValue::ModuleProxy {
        path: resolved.module_path.clone(),
        exports,
        fns,
        native_exports: crate::runtime::NativeFnMap::new(),
        module_env: module_state.env,
    })
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
