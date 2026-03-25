use axum::response::IntoResponse;
use indexmap::IndexMap;
use serde_json::Value as JsonValue;

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
        let mut router = axum::Router::new();

        for route in self.routes {
            let path_str = format!("/{}", route.path.trim_start_matches('/'));
            let method_str = route.method.clone();
            let handler_return_type = route.return_type.clone();
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
                            input.headers.insert(name.as_str().to_string(), v.to_string());
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
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            axum::response::Json(serde_json::json!({"error": err_msg})),
                        )
                            .into_response();
                    }

                    let return_type = handler_return_type.as_deref().unwrap_or("JSON");
                    build_http_response(result, return_type)
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
            router = router.route(
                &path_str,
                axum::routing::MethodRouter::new().on(method, handler),
            );
        }

        for static_route in self.static_routes {
            let dir_path = static_route.module_path.replace('.', "/");
            let static_dir = std::path::Path::new(&static_route.base_dir).join(&dir_path);
            use tower_http::services::fs::ServeDir;
            router = router.nest_service(&static_route.url_prefix, ServeDir::new(static_dir));
        }

        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, router).await.unwrap();
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
        JsonValue::Array(arr) => DolangValue::List(arr.into_iter().map(json_to_dolang_value).collect()),
        JsonValue::Object(obj) => {
            let mut map = IndexMap::new();
            for (k, v) in obj {
                map.insert(k, json_to_dolang_value(v));
            }
            DolangValue::Json(map)
        }
    }
}
