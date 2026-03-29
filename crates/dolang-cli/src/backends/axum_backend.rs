use axum::http::{HeaderName, HeaderValue, Method};
use axum::response::IntoResponse;
use dolang::diagnostics::Severity;
use dolang::diagnostics::{Diagnostic, codes};
use dolang::error::Error;
use dolang::interpreter::CorsConfig;
use indexmap::IndexMap;
use std::collections::HashSet;
use serde_json::Value as JsonValue;
use std::net::TcpListener;
use std::str::FromStr;
use std::time::Duration;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

use dolang::interpreter::{DolangValue, HttpRoute, StaticRoute};
use dolang::runtime::{HandlerInput, RuntimeContext, backend::HttpBackend, execute_http_route};

pub struct AxumBackend {
    routes: Vec<HttpRoute>,
    static_routes: Vec<StaticRoute>,
}

impl AxumBackend {
    pub fn new() -> Self {
        Self {
            routes: vec![],
            static_routes: vec![],
        }
    }
}

impl HttpBackend for AxumBackend {
    fn register_route(&mut self, route: HttpRoute) {
        self.routes.push(route);
    }

    fn register_static(&mut self, route: StaticRoute) {
        self.static_routes.push(route);
    }

    async fn serve(self, context: RuntimeContext, host: &str, port: u16) {
        let addr = format!("{}:{}", host, port);
        let router = match build_router(self.routes, self.static_routes, context) {
            Ok(router) => router,
            Err(err) => {
                eprintln!("{err}");
                return;
            }
        };
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        serve_router(router, listener).await;
    }
}

impl AxumBackend {
    pub async fn serve_with_std_listener(
        self,
        context: RuntimeContext,
        listener: TcpListener,
    ) -> Result<(), Error> {
        listener.set_nonblocking(true).unwrap();
        let listener = tokio::net::TcpListener::from_std(listener).unwrap();
        let router = build_router(self.routes, self.static_routes, context)?;
        serve_router(router, listener).await;
        Ok(())
    }
}

// ─── Response building ────────────────────────────────────────────────────────

fn build_http_response(result: Option<DolangValue>, return_type: &str) -> axum::response::Response {
    match result {
        // $RES(status, body) — status code is now respected
        Some(DolangValue::Response { status, body }) => {
            let json_body = body.map(|b| value_to_json(&b)).unwrap_or(JsonValue::Null);
            let status_code = axum::http::StatusCode::from_u16(status)
                .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            (status_code, axum::response::Json(json_body)).into_response()
        }

        // $HTML() constructor
        Some(DolangValue::Html(html_val)) => {
            let html_content = match *html_val {
                DolangValue::Str(s) => s,
                v => v.to_string(),
            };
            axum::response::Html(html_content).into_response()
        }

        result => {
            // Explicit -> HTML return type
            if return_type == "HTML" {
                let html_content = match result {
                    Some(DolangValue::Str(s)) => s,
                    Some(v) => v.to_string(),
                    None => String::new(),
                };
                return axum::response::Html(html_content).into_response();
            }

            // JSON and everything else
            let json_value = match result {
                Some(DolangValue::Json(map)) | Some(DolangValue::Map(map)) => {
                    let mut obj = serde_json::Map::new();
                    for (k, v) in map {
                        obj.insert(k, value_to_json(&v));
                    }
                    JsonValue::Object(obj)
                }
                Some(DolangValue::Str(s)) => {
                    if return_type == "JSON" {
                        serde_json::from_str::<JsonValue>(&s)
                            .unwrap_or_else(|_| serde_json::json!({"value": s}))
                    } else {
                        serde_json::json!({"value": s})
                    }
                }
                Some(v) => value_to_json(&v),
                None => JsonValue::Object(serde_json::Map::new()),
            };

            axum::response::Json(json_value).into_response()
        }
    }
}

fn append_response_headers(
    mut response: axum::response::Response,
    headers: &[(String, String)],
) -> axum::response::Response {
    let response_headers = response.headers_mut();

    for (name, value) in headers {
        let name = match HeaderName::from_bytes(name.as_bytes()) {
            Ok(name) => name,
            Err(_) => continue,
        };
        let value = match HeaderValue::from_str(value) {
            Ok(value) => value,
            Err(_) => continue,
        };
        response_headers.insert(name, value);
    }

    response
}

async fn serve_router(router: axum::Router, listener: tokio::net::TcpListener) {
    axum::serve(listener, router).await.unwrap();
}

fn build_router(
    routes: Vec<HttpRoute>,
    static_routes: Vec<StaticRoute>,
    context: RuntimeContext,
) -> Result<axum::Router, Error> {
    validate_runtime_context(&context)?;
    let mut router = axum::Router::new();

    for route in routes {
        let path_str = format!("/{}", route.path.trim_start_matches('/'));
        let method_str = route.method.clone();
        let handler_return_type = route.return_type.clone();
        let (resolved_cors, cors_warnings) = resolve_cors(
            route.cors.as_ref(),
            route.parent_cors.as_ref(),
            context.global_cors(),
        )?;
        emit_diagnostics(&cors_warnings);
        let cors_layer = build_cors_layer(resolved_cors.as_ref());
        let (response_headers, header_warnings) = normalize_response_headers(
            &route.response_headers,
            &route.method,
            &path_str,
        );
        emit_diagnostics(&header_warnings);
        let route_definition = route;
        let handler_context = context.clone();

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

                if let Some(err_msg) = error {
                    if err_msg == "exit" {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            axum::response::Json(serde_json::json!({"error": "server stopped"})),
                        )
                            .into_response();
                    }
                    eprintln!("[ERROR] {err_msg}");
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        axum::response::Json(serde_json::json!({"error": err_msg})),
                    )
                        .into_response();
                }

                let return_type = handler_return_type.as_deref().unwrap_or("JSON");
                let response = build_http_response(result, return_type);
                append_response_headers(response, &response_headers)
            }
        };

        use axum::routing::MethodFilter;
        let method = match method_str.as_str() {
            "GET" => MethodFilter::GET,
            "POST" => MethodFilter::POST,
            "PUT" => MethodFilter::PUT,
            "DELETE" => MethodFilter::DELETE,
            "PATCH" => MethodFilter::PATCH,
            _ => continue,
        };
        let method_router = axum::routing::MethodRouter::new().on(method, handler);
        let method_router = if let Some(cors_layer) = cors_layer {
            method_router.layer(cors_layer)
        } else {
            method_router
        };
        router = router.route(&path_str, method_router);
    }

    for static_route in static_routes {
        let dir_path = static_route.module_path.replace('.', "/");
        let static_dir = std::path::Path::new(&static_route.base_dir).join(&dir_path);
        use tower_http::services::fs::ServeDir;
        router = router.nest_service(&static_route.url_prefix, ServeDir::new(static_dir));
    }

    Ok(router)
}

pub fn validate_runtime_context(context: &RuntimeContext) -> Result<(), Error> {
    validate_cors_configs(context.routes(), context.global_cors())
}

fn validate_cors_configs(
    routes: &[HttpRoute],
    global_cors: Option<&CorsConfig>,
) -> Result<(), Error> {
    if let Some(cors) = global_cors {
        validate_cors_config(cors)?;
    }
    for route in routes {
        if let Some(cors) = route.cors.as_ref() {
            validate_cors_config(cors)?;
        }
    }
    Ok(())
}

fn validate_cors_config(cors: &CorsConfig) -> Result<(), Error> {
    let has_wildcard_origin = cors.allow_all || cors.origins.iter().any(|origin| origin == "*");
    if cors.credentials && has_wildcard_origin {
        return Err(Error::from(
            Diagnostic::error(codes::CONFIG_CORS_INVALID, "invalid CORS configuration")
                .with_note("credentials: true cannot be combined with origins: [\"*\"]")
                .with_note("browsers require explicit origins when credentials are enabled"),
        ));
    }
    if let Some(max_age) = cors.max_age
        && max_age < 0
    {
        return Err(Error::from(
            Diagnostic::error(codes::CONFIG_CORS_INVALID, "invalid CORS configuration")
                .with_note("max_age must be a non-negative integer"),
        ));
    }

    Ok(())
}

fn build_cors_layer(cors: Option<&CorsConfig>) -> Option<CorsLayer> {
    let cors = cors?;
    let mut layer = CorsLayer::new();

    layer = if cors.allow_all || cors.origins.iter().any(|origin| origin == "*") {
        layer.allow_origin(AllowOrigin::any())
    } else {
        let origins = cors
            .origins
            .iter()
            .filter_map(|origin| HeaderValue::from_str(origin).ok())
            .collect::<Vec<_>>();
        if origins.is_empty() {
            return None;
        }
        layer.allow_origin(AllowOrigin::list(origins))
    };

    if !cors.methods.is_empty() {
        let methods = cors
            .methods
            .iter()
            .filter_map(|method| Method::from_str(method).ok())
            .collect::<Vec<_>>();
        if !methods.is_empty() {
            layer = layer.allow_methods(AllowMethods::list(methods));
        }
    }

    if !cors.headers.is_empty() {
        let headers = cors
            .headers
            .iter()
            .filter_map(|header| HeaderName::from_str(header).ok())
            .collect::<Vec<_>>();
        if !headers.is_empty() {
            layer = layer.allow_headers(AllowHeaders::list(headers));
        }
    }

    if let Some(max_age) = cors.max_age {
        layer = layer.max_age(Duration::from_secs(max_age as u64));
    }

    if cors.credentials {
        layer = layer.allow_credentials(true);
    }

    Some(layer)
}

fn emit_diagnostics(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        match diagnostic.severity {
            Severity::Warning => eprintln!("{diagnostic}"),
            Severity::Error => eprintln!("{diagnostic}"),
        }
    }
}

fn normalize_response_headers(
    headers: &[(String, String)],
    method: &str,
    path: &str,
) -> (Vec<(String, String)>, Vec<Diagnostic>) {
    let mut warnings = Vec::new();
    let mut normalized = Vec::new();
    let mut seen = HashSet::new();

    for (name, value) in headers {
        let lowered = name.to_ascii_lowercase();
        if !seen.insert(lowered.clone()) {
            warnings.push(
                Diagnostic::warning(
                    codes::CONFIG_HDR_INVALID,
                    format!("duplicate response header overridden: \"{name}\""),
                )
                .with_note(format!("route: {method} {path}"))
                .with_note("later @SET_HDR declarations override earlier ones on the same node"),
            );
            normalized.retain(|(existing_name, _): &(String, String)| {
                existing_name.to_ascii_lowercase() != lowered
            });
        }

        match (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            (Ok(_), Ok(_)) => normalized.push((name.clone(), value.clone())),
            (Err(_), _) => warnings.push(
                Diagnostic::warning(
                    codes::CONFIG_HDR_INVALID,
                    format!("invalid response header name skipped: \"{name}\""),
                )
                .with_note(format!("route: {method} {path}"))
                .with_note("header names must not contain spaces or control characters"),
            ),
            (_, Err(_)) => warnings.push(
                Diagnostic::warning(
                    codes::CONFIG_HDR_INVALID,
                    format!("invalid response header value skipped for: \"{name}\""),
                )
                .with_note(format!("route: {method} {path}")),
            ),
        }
    }

    (normalized, warnings)
}

fn resolve_cors(
    route_cors: Option<&CorsConfig>,
    parent_cors: Option<&CorsConfig>,
    global_cors: Option<&CorsConfig>,
) -> Result<(Option<CorsConfig>, Vec<Diagnostic>), Error> {
    let mut warnings = Vec::new();

    let levels = [
        ("route-level", route_cors),
        ("block-level", parent_cors),
        ("global-level", global_cors),
    ];

    for (index, (level_name, candidate)) in levels.into_iter().enumerate() {
        let Some(candidate) = candidate else {
            continue;
        };

        let (normalized, mut candidate_warnings) = normalize_cors_config(candidate, level_name)?;
        warnings.append(&mut candidate_warnings);
        if let Some(normalized) = normalized {
            return Ok((Some(normalized), warnings));
        }

        if let Some((fallback_name, _)) =
            levels.iter().skip(index + 1).find(|(_, candidate)| candidate.is_some())
        {
            warnings.push(
                Diagnostic::warning(
                    codes::CONFIG_CORS_INVALID,
                    format!(
                        "{level_name} @CORS config is empty after validation, falling back to {fallback_name} config"
                    ),
                ),
            );
        }
    }

    Ok((None, warnings))
}

fn normalize_cors_config(
    cors: &CorsConfig,
    level_name: &str,
) -> Result<(Option<CorsConfig>, Vec<Diagnostic>), Error> {
    validate_cors_config(cors)?;

    if cors.allow_all || cors.origins.iter().any(|origin| origin == "*") {
        let mut normalized = cors.clone();
        normalized.allow_all = true;
        normalized.origins.clear();
        return Ok((Some(normalized), Vec::new()));
    }

    let mut warnings = Vec::new();
    let mut origins = Vec::new();
    for origin in &cors.origins {
        if is_valid_origin(origin) {
            origins.push(origin.clone());
        } else {
            warnings.push(
                Diagnostic::warning(
                    codes::CONFIG_CORS_INVALID,
                    format!("invalid origin URL skipped: \"{origin}\""),
                )
                .with_note(format!("in {level_name} @CORS"))
                .with_note("only URLs with http:// or https:// scheme are allowed"),
            );
        }
    }

    let mut methods = Vec::new();
    for method in &cors.methods {
        if is_supported_method(method) {
            methods.push(method.clone());
        } else {
            warnings.push(
                Diagnostic::warning(
                    codes::CONFIG_CORS_INVALID,
                    format!("invalid HTTP method skipped: \"{method}\""),
                )
                .with_note(format!("in {level_name} @CORS")),
            );
        }
    }

    let mut headers = Vec::new();
    for header in &cors.headers {
        if HeaderName::from_str(header).is_ok() {
            headers.push(header.clone());
        } else {
            warnings.push(
                Diagnostic::warning(
                    codes::CONFIG_CORS_INVALID,
                    format!("invalid request header name skipped: \"{header}\""),
                )
                .with_note(format!("in {level_name} @CORS")),
            );
        }
    }

    if origins.is_empty() {
        warnings.push(
            Diagnostic::warning(
                codes::CONFIG_CORS_INVALID,
                "CORS origins list is empty after validation".to_string(),
            )
            .with_note(format!("in {level_name} @CORS")),
        );
        return Ok((None, warnings));
    }

    Ok((
        Some(CorsConfig {
            allow_all: false,
            origins,
            methods,
            headers,
            max_age: cors.max_age,
            credentials: cors.credentials,
        }),
        warnings,
    ))
}

fn is_valid_origin(origin: &str) -> bool {
    (origin.starts_with("http://") || origin.starts_with("https://"))
        && HeaderValue::from_str(origin).is_ok()
}

fn is_supported_method(method: &str) -> bool {
    matches!(
        method,
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cors(origins: &[&str], methods: &[&str], headers: &[&str]) -> CorsConfig {
        CorsConfig {
            allow_all: false,
            origins: origins.iter().map(|v| (*v).to_string()).collect(),
            methods: methods.iter().map(|v| (*v).to_string()).collect(),
            headers: headers.iter().map(|v| (*v).to_string()).collect(),
            max_age: None,
            credentials: false,
        }
    }

    #[test]
    fn normalize_response_headers_skips_invalid_names_and_warns() {
        let (headers, warnings) = normalize_response_headers(
            &[
                ("bad header".to_string(), "ignored".to_string()),
                ("X-Valid".to_string(), "visible".to_string()),
            ],
            "GET",
            "/health",
        );

        assert_eq!(headers, vec![("X-Valid".to_string(), "visible".to_string())]);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, codes::CONFIG_HDR_INVALID);
        assert_eq!(warnings[0].severity, Severity::Warning);
    }

    #[test]
    fn normalize_response_headers_duplicate_names_warn_and_last_wins() {
        let (headers, warnings) = normalize_response_headers(
            &[
                ("X-Test".to_string(), "1".to_string()),
                ("x-test".to_string(), "2".to_string()),
            ],
            "GET",
            "/health",
        );

        assert_eq!(headers, vec![("x-test".to_string(), "2".to_string())]);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, codes::CONFIG_HDR_INVALID);
    }

    #[test]
    fn resolve_cors_filters_invalid_entries_and_falls_back() {
        let route = cors(&["not-a-url"], &["GET"], &[]);
        let parent = cors(&["https://parent.example.com"], &["GET"], &[]);

        let (resolved, warnings) = resolve_cors(Some(&route), Some(&parent), None).expect("cors");

        assert_eq!(
            resolved.expect("resolved").origins,
            vec!["https://parent.example.com".to_string()]
        );
        assert!(warnings.iter().any(|warning| warning.code == codes::CONFIG_CORS_INVALID));
    }

    #[test]
    fn resolve_cors_filters_invalid_methods_and_headers() {
        let config = cors(
            &["https://client.example.com"],
            &["GET", "NOPE"],
            &["Authorization", "bad header"],
        );

        let (resolved, warnings) = resolve_cors(Some(&config), None, None).expect("cors");
        let resolved = resolved.expect("resolved");
        assert_eq!(resolved.methods, vec!["GET".to_string()]);
        assert_eq!(resolved.headers, vec!["Authorization".to_string()]);
        assert_eq!(warnings.len(), 2);
    }
}

fn value_to_json(v: &DolangValue) -> JsonValue {
    match v {
        DolangValue::Null => JsonValue::Null,
        DolangValue::Bool(b) => JsonValue::Bool(*b),
        DolangValue::Int(i) => JsonValue::Number(serde_json::Number::from(*i)),
        DolangValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        DolangValue::Str(s) => JsonValue::String(s.clone()),
        DolangValue::List(list) => JsonValue::Array(list.iter().map(value_to_json).collect()),
        DolangValue::Json(map) | DolangValue::Map(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                obj.insert(k.clone(), value_to_json(v));
            }
            JsonValue::Object(obj)
        }
        DolangValue::TypedInstance { fields, .. } => {
            let mut obj = serde_json::Map::new();
            for (k, v) in fields {
                if !k.starts_with('_') {
                    obj.insert(k.clone(), value_to_json(v));
                }
            }
            JsonValue::Object(obj)
        }
        _ => JsonValue::Null,
    }
}

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
            DolangValue::List(arr.into_iter().map(json_to_dolang_value).collect())
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
