// HTTP Server - handles dolang serve mode with Axum
use std::collections::HashMap;
use std::process;

use axum::response::IntoResponse;
use indexmap::IndexMap;
use serde_json::Value as JsonValue;

use dolang::interpreter::env::ValueType;
use dolang::interpreter::{
    DolangValue, FnEnv, STATIC_ROUTES, exec_http_handler, get_routes, print_routes, set_current_file,
};
use dolang::parser;

/// Start HTTP server with the given path (main.dol file or directory)
pub fn run_serve(path: std::path::PathBuf, show_routertab: bool) {
    use dolang::config;

    // Determine the serve directory and main file
    let serve_dir = if path.is_file() {
        path.parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf()
    } else {
        path.clone()
    };

    // Load project config if exists
    if let Some(loaded_config) = config::ProjectConfig::load_from_dir(&serve_dir) {
        println!(
            "Loaded project: {} v{}",
            loaded_config.name, loaded_config.version
        );
        config::set_serve_config(loaded_config);
    }

    // Find main.dol
    let main_file = if path.is_file() {
        path
    } else {
        path.join("main.dol")
    };

    let main_path = main_file.to_string_lossy().to_string();

    if !std::path::Path::new(&main_path).exists() {
        eprintln!("[ERROR] main file not found: {}", main_path);
        process::exit(1);
    }

    println!("Running serve mode: {}", main_path);
    println!("Use $main() {{ ... }} to define server logic");
    println!();

    // Set current file to "main.dol" for $main() scope checking
    // Use the serve_dir as the base directory for static file resolution
    let main_dol_path = serve_dir.join("main.dol");
    set_current_file(Some(main_dol_path.to_string_lossy().to_string()));

    // Read and parse the main file
    let content = match std::fs::read_to_string(&main_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERROR] cannot read file '{}': {}", main_path, e);
            process::exit(1);
        }
    };

    let statements = match parser::parse(&content) {
        Ok(stmts) => stmts,
        Err(e) => {
            eprintln!("[ERROR] {}", e);
            process::exit(1);
        }
    };

    // Execute statements (register routes)
    let mut env: HashMap<String, DolangValue> = HashMap::new();
    let mut type_env: HashMap<String, ValueType> = HashMap::new();
    let mut const_env: HashMap<String, bool> = HashMap::new();
    let mut fns: FnEnv = HashMap::new();

    for stmt in statements {
        let (cont, exec_err) =
            dolang::interpreter::exec(&stmt, &mut env, &mut fns, &mut type_env, &mut const_env);
        if let Err(e) = exec_err {
            eprintln!("[ERROR] {}", e);
            process::exit(1);
        }
        if !cont {
            process::exit(0);
        }
    }

    // Start HTTP server if routes are registered
    let routes = get_routes();

    if !routes.is_empty() {
        // Get server config
        let host = config::get_serve_config()
            .map(|c| c.server.host.clone())
            .unwrap_or_else(|| "0.0.0.0".to_string());
        let port = config::get_serve_config()
            .map(|c| c.server.port)
            .unwrap_or(8080);

        let addr = format!("{}:{}", host, port);

        // Print routes table if --routertab is enabled
        if show_routertab {
            println!();
            print_routes();
            println!();
        }

        println!("Server running at http://{}", addr);

        // Build Axum router with routes
        let mut router = axum::Router::new();

        // Use indexed loop to avoid borrowing issues
        let routes_count = routes.len();
        for i in 0..routes_count {
            let route = &routes[i];
            if !show_routertab {
                println!("  {} {} -> {}", route.method, route.path, route.name);
            }

            // Create handler closure for each route - clone all data needed
            let handler_body = route.body.clone();
            let handler_params = route.params.clone();
            let handler_return_type = route.return_type.clone();
            let handler_method = route.method.clone();
            let route_path = route.path.clone();
            let handler_params_for_closure = handler_params.clone();
            let handler_method_for_closure = handler_method.clone();
            let route_path_for_closure = route_path.clone();

            // Use a simpler approach - extract all from request
            let handler = move |req: axum::extract::Request| {
                async move {
                    // Build environment for handler execution
                    let mut env: HashMap<String, DolangValue> = HashMap::new();
                    let mut type_env: HashMap<String, ValueType> = HashMap::new();
                    let mut const_env: HashMap<String, bool> = HashMap::new();
                    let mut fns: FnEnv = HashMap::new();

                    // Extract path from URL
                    let uri = req.uri();
                    let url_path = uri.path();

                    // Parse route path to extract param definitions (inside closure)
                    let route_seg_parts: Vec<&str> = route_path_for_closure.split('/').collect();
                    let url_seg_parts: Vec<&str> = url_path.split('/').collect();

                    // Match path params by position
                    let route_len = route_seg_parts.len();
                    let url_len = url_seg_parts.len();

                    if route_len == url_len {
                        for idx in 0..route_len {
                            let route_seg = route_seg_parts[idx];
                            let url_seg = url_seg_parts[idx];
                            if route_seg.starts_with(':') {
                                let param_name = &route_seg[1..];
                                env.insert(
                                    param_name.to_string(),
                                    DolangValue::Str(url_seg.to_string()),
                                );
                            } else if idx < handler_params_for_closure.len() {
                                let param_name = &handler_params_for_closure[idx];
                                env.insert(
                                    param_name.clone(),
                                    DolangValue::Str(url_seg.to_string()),
                                );
                            }
                        }
                    }

                    // Extract query parameters
                    if let Some(query) = uri.query() {
                        for pair in query.split('&') {
                            let parts: Vec<&str> = pair.split('=').collect();
                            if parts.len() == 2 {
                                env.insert(
                                    parts[0].to_string(),
                                    DolangValue::Str(parts[1].to_string()),
                                );
                            }
                        }
                    }

                    // Extract body for POST/PUT/PATCH
                    // Note: we need to get headers before consuming the body
                    let is_body_method = matches!(
                        handler_method_for_closure.as_str(),
                        "POST" | "PUT" | "PATCH"
                    );

                    // Extract HTTP headers into a Map
                    let headers = req.headers();
                    let mut header_map: IndexMap<String, DolangValue> = IndexMap::new();
                    for (name, value) in headers {
                        if let Ok(v) = value.to_str() {
                            header_map
                                .insert(name.as_str().to_string(), DolangValue::Str(v.to_string()));
                        }
                    }
                    env.insert("__headers__".to_string(), DolangValue::Json(header_map));

                    if is_body_method {
                        let body_bytes = axum::body::to_bytes(req.into_body(), 1024 * 1024)
                            .await
                            .unwrap_or_default();
                        if !body_bytes.is_empty() {
                            if let Ok(json) = serde_json::from_slice::<JsonValue>(&body_bytes) {
                                let dolang_body = json_to_dolang_value(json);
                                env.insert("body".to_string(), dolang_body);
                            }
                        }
                    }

                    // Execute handler body using exec_http_handler
                    let (_should_continue, result, error) = exec_http_handler(
                        &handler_body,
                        &mut env,
                        &mut fns,
                        &mut type_env,
                        &mut const_env,
                    );

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
                    let response = build_http_response(result, return_type);

                    response
                }
            };

            // Register route in Axum
            use axum::routing::MethodFilter;
            let path = format!("/{}", route_path.trim_start_matches('/'));
            let method = match handler_method.as_str() {
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
        let static_routes = {
            let routes = STATIC_ROUTES.lock().unwrap();
            routes.clone()
        };

        if !static_routes.is_empty() {
            for static_route in &static_routes {
                if !show_routertab {
                    println!("  STATIC {} -> {}", static_route.url_prefix, static_route.module_path);
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
