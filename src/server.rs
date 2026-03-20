// HTTP Server - handles dolang serve mode with Axum
use std::process;

use axum::response::IntoResponse;
use indexmap::IndexMap;
use serde_json::Value as JsonValue;

use dolang::interpreter::DolangValue;
use dolang::runtime::{
    HandlerInput, ProgramState, RuntimeMode, execute_http_route, execute_program_with_writer,
    load_context_and_program,
};

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
    println!("Use $main() {{ ... }} to define server logic");
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

    // Start HTTP server if routes are registered
    let routes = context.routes().to_vec();

    if !routes.is_empty() {
        let host = context.server_host().to_string();
        let port = context.server_port();
        let addr = format!("{}:{}", host, port);

        // Print routes table if --routertab is enabled
        if show_routertab {
            println!();
            context.print_routes();
            println!();
        }

        println!("Server running at http://{}", addr);

        // Build Axum router with routes
        let mut router = axum::Router::new();

        // Use indexed loop to avoid borrowing issues
        for route in &routes {
            if !show_routertab {
                println!("  {} {} -> {}", route.method, route.path, route.name);
            }

            // Create handler closure for each route - clone all data needed
            let handler_return_type = route.return_type.clone();
            let route_definition = route.clone();
            let handler_context = context.clone();

            // Use a simpler approach - extract all from request
            let handler = move |req: axum::extract::Request| {
                let route_definition = route_definition.clone();
                let handler_context = handler_context.clone();
                async move {
                    let uri = req.uri();
                    let mut input = HandlerInput::new(uri.path());
                    input.query = uri.query().map(str::to_string);

                    for (name, value) in req.headers() {
                        if let Ok(v) = value.to_str() {
                            input
                                .headers
                                .insert(name.as_str().to_string(), v.to_string());
                        }
                    }

                    if matches!(route_definition.method.as_str(), "POST" | "PUT" | "PATCH") {
                        let body_bytes = axum::body::to_bytes(req.into_body(), 1024 * 1024)
                            .await
                            .unwrap_or_default();
                        if !body_bytes.is_empty()
                            && let Ok(json) = serde_json::from_slice::<JsonValue>(&body_bytes)
                        {
                            input.body = Some(json_to_dolang_value(json));
                        }
                    }

                    let (_should_continue, result, error) =
                        execute_http_route(&route_definition, &input, &handler_context);

                    // Handle error
                    if let Some(err_msg) = error {
                        if err_msg == "exit" {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                axum::response::Json(
                                    serde_json::json!({"error": "server stopped"}),
                                ),
                            )
                                .into_response();
                        }
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            axum::response::Json(serde_json::json!({
                                "error": err_msg
                            })),
                        )
                            .into_response();
                    }

                    // Build response based on return value and return type
                    let return_type = handler_return_type.as_deref().unwrap_or("JSON");
                    build_http_response(result, return_type)
                }
            };

            // Register route in Axum
            use axum::routing::MethodFilter;
            let path = format!("/{}", route.path.trim_start_matches('/'));
            let method = match route.method.as_str() {
                "GET" => MethodFilter::GET,
                "POST" => MethodFilter::POST,
                "PUT" => MethodFilter::PUT,
                "DELETE" => MethodFilter::DELETE,
                "PATCH" => MethodFilter::PATCH,
                _ => continue,
            };
            router = router.route(
                &path,
                axum::routing::MethodRouter::new().on(method, handler),
            );
        }

        // Add static file routes
        let static_routes = context.static_routes().to_vec();

        if !static_routes.is_empty() {
            for static_route in &static_routes {
                if !show_routertab {
                    println!(
                        "  STATIC {} -> {}",
                        static_route.url_prefix, static_route.module_path
                    );
                }

                // Create static file handler
                let url_prefix = static_route.url_prefix.clone();
                let module_path = static_route.module_path.clone();
                let base_dir = static_route.base_dir.clone();

                // Add static route using tower's ServeDir
                // module_path is like "css" or "css.dolang" -> we need to convert it to directory
                let dir_path = module_path.replace('.', "/");
                let static_dir = std::path::Path::new(&base_dir).join(&dir_path);

                use tower_http::services::fs::ServeDir;
                let serve_dir = ServeDir::new(static_dir);
                router = router.nest_service(&url_prefix, serve_dir);
            }
        }

        // Start server
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            axum::serve(listener, router).await.unwrap();
        });
    } else {
        println!("No HTTP routes registered.");
    }
}

// ─── Helper functions for HTTP response ───────────────────────────────────────

/// Convert serde_json::Value to DolangValue
fn json_to_dolang_value(json: JsonValue) -> DolangValue {
    match json {
        JsonValue::Null => DolangValue::Null,
        JsonValue::Bool(b) => DolangValue::Bool(b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                DolangValue::Int(i)
            } else if let Some(f) = n.as_f64() {
                DolangValue::Float(f)
            } else {
                DolangValue::Null
            }
        }
        JsonValue::String(s) => DolangValue::Str(s),
        JsonValue::Array(arr) => {
            let list: Vec<DolangValue> = arr.into_iter().map(json_to_dolang_value).collect();
            DolangValue::List(list)
        }
        JsonValue::Object(obj) => {
            let mut map = IndexMap::new();
            for (k, v) in obj {
                map.insert(k, json_to_dolang_value(v));
            }
            DolangValue::Json(map)
        }
    }
}

/// Build HTTP response based on return value and return type
fn build_http_response(result: Option<DolangValue>, return_type: &str) -> axum::response::Response {
    // Handle $HTML() constructor response - more "interpretive" style
    if let Some(DolangValue::Html(html_val)) = &result {
        let html_content = match &**html_val {
            DolangValue::Str(s) => s.clone(),
            v => v.to_string(),
        };
        return axum::response::Html(html_content).into_response();
    }

    // Handle explicit -> HTML return type
    if return_type == "HTML" {
        let html_content = match result {
            Some(DolangValue::Str(s)) => s,
            Some(v) => v.to_string(),
            None => String::new(),
        };
        return axum::response::Html(html_content).into_response();
    }

    // Handle JSON and other types
    let json_value = match result {
        Some(DolangValue::Response { status: _, body }) => {
            body.map(|b| value_to_json(&b)).unwrap_or(JsonValue::Null)
        }
        Some(DolangValue::Json(map)) => {
            let mut obj: serde_json::Map<String, JsonValue> = serde_json::Map::new();
            for item in map.iter() {
                let k: &String = item.0;
                let v: &DolangValue = item.1;
                obj.insert(k.to_string(), value_to_json(v));
            }
            JsonValue::Object(obj)
        }
        Some(DolangValue::Str(s)) => {
            if return_type == "JSON" {
                if let Ok(v) = serde_json::from_str::<JsonValue>(&s) {
                    v
                } else {
                    serde_json::json!({"value": s})
                }
            } else {
                serde_json::json!({"value": s})
            }
        }
        Some(v) => value_to_json(&v),
        None => JsonValue::Object(serde_json::Map::new()),
    };

    axum::response::Json(json_value).into_response()
}

/// Convert DolangValue to serde_json::Value
fn value_to_json(v: &DolangValue) -> JsonValue {
    match v {
        DolangValue::Null => JsonValue::Null,
        DolangValue::Bool(b) => JsonValue::Bool(*b),
        DolangValue::Int(i) => JsonValue::Number(serde_json::Number::from(*i)),
        DolangValue::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(*f) {
                JsonValue::Number(n)
            } else {
                JsonValue::Null
            }
        }
        DolangValue::Str(s) => JsonValue::String(s.clone()),
        DolangValue::List(list) => {
            let arr: Vec<JsonValue> = list.iter().map(value_to_json).collect();
            JsonValue::Array(arr)
        }
        DolangValue::Json(map) => {
            let mut obj: serde_json::Map<String, JsonValue> = serde_json::Map::new();
            for item in map.iter() {
                let k: &String = item.0;
                let v: &DolangValue = item.1;
                obj.insert(k.to_string(), value_to_json(v));
            }
            JsonValue::Object(obj)
        }
        _ => JsonValue::Null,
    }
}
