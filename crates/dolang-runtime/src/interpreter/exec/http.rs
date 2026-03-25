use std::fs;
use std::io::Write;
use std::path::Path;

use crate::ast::{HttpBlockStmt, HttpFnStmt, Stmt};
use crate::error::Error;
use crate::module::ModuleResolver;
use crate::runtime::RuntimeContext;

use super::super::HttpRoute;
use super::Flow;

pub(super) fn handle_http_fn(
    stmt: &HttpFnStmt,
    context: &mut RuntimeContext,
    w: &mut dyn Write,
) -> Flow {
    let route = HttpRoute {
        method: stmt.method.clone(),
        path: stmt.path.clone(),
        name: stmt.name.clone(),
        params: stmt.params.clone(),
        variadic_param: stmt.variadic_param.clone(),
        return_type: stmt.return_type.clone(),
        body: stmt.body.clone(),
    };

    context.register_http_route(route);
    writeln!(
        w,
        "[INFO] HTTP route registered: {} {}",
        stmt.method, stmt.path
    )
    .ok();
    Flow::Normal
}

pub(super) fn handle_http_block(
    stmt: &HttpBlockStmt,
    context: &mut RuntimeContext,
    w: &mut dyn Write,
) -> Flow {
    if let Some(link_module) = &stmt.link {
        let prefix = stmt.prefix.as_deref().unwrap_or("");
        let module_routes = match load_module_routes(link_module, context) {
            Ok(routes) => routes,
            Err(err) => return Flow::Err(err),
        };

        let route_count = module_routes.len();
        for route in module_routes {
            let prefix_trimmed = prefix.trim_start_matches('/').trim_end_matches('/');
            let path_trimmed = route.path.trim_start_matches('/');
            let full_path = if prefix_trimmed.is_empty() {
                format!("/{}", path_trimmed)
            } else {
                format!("/{}/{}", prefix_trimmed, path_trimmed)
            };
            context.register_http_route(HttpRoute {
                method: route.method.clone(),
                path: full_path,
                name: route.name.clone(),
                params: route.params.clone(),
                variadic_param: route.variadic_param.clone(),
                return_type: route.return_type.clone(),
                body: route.body.clone(),
            });
        }

        writeln!(
            w,
            "[INFO] HTTP linked module '{}' with prefix '{}' ({} routes)",
            link_module, prefix, route_count
        )
        .ok();
        return Flow::Normal;
    }

    for route_stmt in &stmt.routes {
        let full_path = if let Some(prefix) = &stmt.prefix {
            let prefix_trimmed = prefix.trim_start_matches('/').trim_end_matches('/');
            let path_trimmed = route_stmt.path.trim_start_matches('/');
            if prefix_trimmed.is_empty() {
                format!("/{}", path_trimmed)
            } else {
                format!("/{}/{}", prefix_trimmed, path_trimmed)
            }
        } else {
            route_stmt.path.clone()
        };

        context.register_http_route(HttpRoute {
            method: route_stmt.method.clone(),
            path: full_path,
            name: route_stmt.name.clone(),
            params: route_stmt.params.clone(),
            variadic_param: route_stmt.variadic_param.clone(),
            return_type: route_stmt.return_type.clone(),
            body: route_stmt.body.clone(),
        });
    }

    writeln!(
        w,
        "[INFO] HTTP block registered {} routes",
        stmt.routes.len()
    )
    .ok();
    Flow::Normal
}

fn load_module_routes(
    module_path: &str,
    context: &RuntimeContext,
) -> Result<Vec<HttpRoute>, Error> {
    use crate::parser;

    let resolver = ModuleResolver::new(context.project_root().to_path_buf())
        .with_current_file(context.current_file().map(Path::new).map(Path::to_path_buf))
        .with_manifest(context.project_config().cloned());

    let resolved = match resolver.resolve_module(module_path) {
        Some(resolved) => resolved,
        None => {
            return Err(Error::Interpreter(format!(
                "module file not found: {} (tried: {})",
                module_path,
                resolver.describe_search_order(module_path)
            )));
        }
    };

    let content = match fs::read_to_string(&resolved.file_path) {
        Ok(content) => content,
        Err(err) => {
            return Err(Error::Interpreter(format!(
                "cannot read module file '{}': {}",
                resolved.file_path.display(),
                err
            )));
        }
    };

    let statements = match parser::parse(&content) {
        Ok(stmts) => stmts,
        Err(err) => {
            return Err(Error::Interpreter(format!(
                "failed to parse module '{}': {}",
                module_path, err
            )));
        }
    };

    let mut routes = Vec::new();
    collect_http_routes(&statements, "", context, &mut routes)?;

    if routes.is_empty() {
        return Err(Error::Interpreter(format!(
            "module '{}' does not define any HTTP routes",
            module_path
        )));
    }

    Ok(routes)
}

fn collect_http_routes(
    statements: &[Stmt],
    base_prefix: &str,
    context: &RuntimeContext,
    routes: &mut Vec<HttpRoute>,
) -> Result<(), Error> {
    for stmt in statements {
        match stmt {
            Stmt::HttpFn(http_fn) => routes.push(http_route_with_prefix(http_fn, base_prefix)),
            Stmt::HttpBlock(http_block) => {
                let next_prefix = match &http_block.prefix {
                    Some(prefix) => join_route_paths(base_prefix, prefix),
                    None => normalize_prefix(base_prefix),
                };

                for route in &http_block.routes {
                    routes.push(http_route_with_prefix(route, &next_prefix));
                }

                if let Some(link_module) = &http_block.link {
                    let linked_routes = load_module_routes(link_module, context)?;
                    for route in linked_routes {
                        routes.push(HttpRoute {
                            path: join_route_paths(&next_prefix, &route.path),
                            ..route
                        });
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn http_route_with_prefix(route: &HttpFnStmt, prefix: &str) -> HttpRoute {
    HttpRoute {
        method: route.method.clone(),
        path: join_route_paths(prefix, &route.path),
        name: route.name.clone(),
        params: route.params.clone(),
        variadic_param: route.variadic_param.clone(),
        return_type: route.return_type.clone(),
        body: route.body.clone(),
    }
}

fn join_route_paths(prefix: &str, path: &str) -> String {
    let normalized_prefix = normalize_prefix(prefix);
    let path_trimmed = path.trim();
    if normalized_prefix.is_empty() {
        if path_trimmed.is_empty() || path_trimmed == "/" {
            "/".to_string()
        } else {
            format!("/{}", path_trimmed.trim_start_matches('/'))
        }
    } else if path_trimmed.is_empty() || path_trimmed == "/" {
        normalized_prefix
    } else {
        format!(
            "{}/{}",
            normalized_prefix.trim_end_matches('/'),
            path_trimmed.trim_start_matches('/')
        )
    }
}

fn normalize_prefix(prefix: &str) -> String {
    let trimmed = prefix.trim();
    if trimmed.is_empty() || trimmed == "/" {
        String::new()
    } else {
        format!("/{}", trimmed.trim_matches('/'))
    }
}
