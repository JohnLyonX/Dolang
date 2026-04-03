// HTTP Server entry point — delegates all Axum logic to AxumBackend
use std::process;

use dolang::runtime::{
    ProgramState, RuntimeMode, backend::HttpBackend, execute_program_with_writer,
    load_context_and_program,
};

use crate::backends::axum_backend::{AxumBackend, validate_runtime_context};

/// Start HTTP server with the given path (main.dol file or directory)
pub fn run_serve(path: std::path::PathBuf, show_routertab: bool) {
    let (mut context, program) = match load_context_and_program(RuntimeMode::Serve, &path) {
        Ok(loaded) => loaded,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    if let Some(loaded_config) = context.project_config() {
        println!(
            "Loaded project: {} v{}",
            loaded_config.name, loaded_config.version
        );
    }

    println!("Running serve mode: {}", program.path.display());
    println!();

    let mut state = ProgramState::new();
    match execute_program_with_writer(
        &program.statements,
        &mut state,
        &mut context,
        &mut std::io::stdout(),
    ) {
        Ok(true) => {}
        Ok(false) => process::exit(0),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }

    if context.routes().is_empty() {
        println!("No HTTP routes registered.");
        return;
    }

    if let Err(err) = validate_runtime_context(&context) {
        eprintln!("{err}");
        process::exit(1);
    }

    let host = context.server_host().to_string();
    let port = context.server_port();

    if show_routertab {
        println!();
        context.print_routes();
        println!();
    } else {
        for route in context.routes() {
            println!("  {} {} -> {}", route.method, route.path, route.name);
        }
        for sr in context.static_routes() {
            println!("  STATIC {} -> {}", sr.url_prefix, sr.module_path);
        }
    }

    println!("Server running at http://{}:{}", host, port);

    // Hand off all routes to the backend
    let mut backend = AxumBackend::new();
    for route in context.routes().iter().cloned() {
        backend.register_route(route);
    }
    for sr in context.static_routes().iter().cloned() {
        backend.register_static(sr);
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(backend.serve(context, &host, port));
}
