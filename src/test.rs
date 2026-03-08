// HTTP Test Runner - handles dolang test mode
use std::collections::HashMap;
use std::process;

use indexmap::IndexMap;
use serde_json::Value as JsonValue;

use dolang::interpreter::{exec_http_handler, get_routes, clear_routes, set_current_file, DolangValue, FnEnv, HttpRoute};
use dolang::interpreter::env::ValueType;
use dolang::parser;

/// Test configuration (from CLI)
pub struct TestConfig {
    pub file: Option<String>,
    pub method: Option<String>,
    pub path: Option<String>,
    pub body: Option<String>,
}

/// Test result
#[derive(Debug)]
pub struct TestResult {
    pub method: String,
    pub path: String,
    pub status: u16,
    pub body: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Run tests with the given configuration
pub fn run_test(config: TestConfig) {
    

    // Determine the file to test
    let test_file = config.file.clone().unwrap_or_else(|| "main.dol".to_string());

    // Check if file exists
    if !std::path::Path::new(&test_file).exists() {
        eprintln!("[ERROR] Test file not found: {}", test_file);
        process::exit(1);
    }

    println!("Running tests from: {}", test_file);
    println!();

    // Clear any existing routes
    clear_routes();

    // Set current file for $main() scope checking
    set_current_file(Some("main.dol".to_string()));

    // Read and parse the test file
    let content = match std::fs::read_to_string(&test_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERROR] cannot read file '{}': {}", test_file, e);
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
        let (cont, exec_err) = dolang::interpreter::exec(&stmt, &mut env, &mut fns, &mut type_env, &mut const_env);
        if let Err(e) = exec_err {
            eprintln!("[ERROR] {}", e);
            process::exit(1);
        }
        if !cont {
            process::exit(0);
        }
    }

    // Get registered routes
    let routes = get_routes();

    if routes.is_empty() {
        println!("No routes registered for testing.");
        process::exit(0);
    }

    // If --route specified, run specific test
    if let (Some(method), Some(path)) = (&config.method, &config.path) {
        let result = run_route_test(&routes, method, path, config.body.as_deref());
        print_test_result(&result);
        if !result.success {
            process::exit(1);
        }
        return;
    }

    // Otherwise, run all registered routes
    println!("Testing {} route(s):", routes.len());
    println!();

    let mut all_passed = true;
    for route in &routes {
        let result = run_route_test(&routes, &route.method, &route.path, None);
        print_test_result(&result);
        if !result.success {
            all_passed = false;
        }
    }

    if all_passed {
        println!();
        println!("All tests passed!");
    } else {
        println!();
        println!("Some tests failed!");
        process::exit(1);
    }
}

/// Run a test for a specific route
fn run_route_test(routes: &[HttpRoute], method: &str, path: &str, body: Option<&str>) -> TestResult {
    // Find matching route
    let route = match routes.iter().find(|r| r.method == method && r.path == path) {
        Some(r) => r,
        None => {
            return TestResult {
                method: method.to_string(),
                path: path.to_string(),
                status: 404,
                body: String::new(),
                success: false,
                error: Some(format!("Route {} {} not found", method, path)),
            };
        }
    };

    // Build environment for handler execution
    let mut env: HashMap<String, DolangValue> = HashMap::new();
    let mut type_env: HashMap<String, ValueType> = HashMap::new();
    let mut const_env: HashMap<String, bool> = HashMap::new();
    let mut fns: FnEnv = HashMap::new();

    // Extract path params
    let route_seg_parts: Vec<&str> = route.path.split('/').collect();
    let url_seg_parts: Vec<&str> = path.split('/').collect();

    if route_seg_parts.len() == url_seg_parts.len() {
        for idx in 0..route_seg_parts.len() {
            let route_seg = route_seg_parts[idx];
            let url_seg = url_seg_parts[idx];
            if route_seg.starts_with(':') {
                let param_name = &route_seg[1..];
                env.insert(param_name.to_string(), DolangValue::Str(url_seg.to_string()));
            }
        }
    }

    // Add body if provided
    if let Some(body_content) = body {
        env.insert("body".to_string(), DolangValue::Str(body_content.to_string()));
    }

    // Add empty headers
    let header_map: IndexMap<String, DolangValue> = IndexMap::new();
    env.insert("__headers__".to_string(), DolangValue::Json(header_map));

    // Execute handler
    let (should_continue, result, error) = exec_http_handler(
        &route.body,
        &mut env,
        &mut fns,
        &mut type_env,
        &mut const_env,
    );

    // Handle error
    if let Some(err_msg) = error {
        return TestResult {
            method: method.to_string(),
            path: path.to_string(),
            status: 500,
            body: String::new(),
            success: false,
            error: Some(err_msg),
        };
    }

    if !should_continue {
        return TestResult {
            method: method.to_string(),
            path: path.to_string(),
            status: 500,
            body: String::new(),
            success: false,
            error: Some("Handler execution failed".to_string()),
        };
    }

    // Build response
    let return_type = route.return_type.clone().unwrap_or_else(|| "JSON".to_string());
    let response_body = build_response(result, &return_type);

    TestResult {
        method: method.to_string(),
        path: path.to_string(),
        status: 200,
        body: response_body,
        success: true,
        error: None,
    }
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

/// Build HTTP response from handler result
fn build_response(result: Option<DolangValue>, return_type: &str) -> String {
    match result {
        Some(val) => {
            match return_type {
                "String" => {
                    match val {
                        DolangValue::Str(s) => s,
                        _ => val.to_string(),
                    }
                }
                "JSON" | "json" => {
                    let json_val = value_to_json(&val);
                    serde_json::to_string(&json_val).unwrap_or_else(|_| "{}".to_string())
                }
                _ => val.to_string(),
            }
        }
        None => String::new(),
    }
}

/// Print test result
fn print_test_result(result: &TestResult) {
    print!("  {} {} => ", result.method, result.path);

    if result.success {
        println!("{} {}", result.status, result.body);
    } else {
        println!("FAILED");
        if let Some(err) = &result.error {
            println!("    Error: {}", err);
        }
    }
}
