use indexmap::IndexMap;

use crate::error::Error;
use crate::interpreter::{DolangValue, HttpRoute, exec_http_handler};

use super::{ProgramState, RuntimeContext};

#[derive(Debug, Clone)]
pub struct HandlerInput {
    pub request_path: String,
    pub query: Option<String>,
    pub headers: IndexMap<String, String>,
    pub body: Option<DolangValue>,
}

impl HandlerInput {
    pub fn new(request_path: impl Into<String>) -> Self {
        Self {
            request_path: request_path.into(),
            query: None,
            headers: IndexMap::new(),
            body: None,
        }
    }
}

pub fn execute_http_route(
    route: &HttpRoute,
    input: &HandlerInput,
    context: &RuntimeContext,
) -> (bool, Option<DolangValue>, Option<String>) {
    let mut runtime_context = context.clone();
    let mut state = ProgramState::new();

    let route_seg_parts: Vec<&str> = route.path.split('/').collect();
    let url_seg_parts: Vec<&str> = input.request_path.split('/').collect();

    if route_seg_parts.len() == url_seg_parts.len() {
        for idx in 0..route_seg_parts.len() {
            let route_seg = route_seg_parts[idx];
            let url_seg = url_seg_parts[idx];
            if let Some(param_name) = route_seg.strip_prefix(':') {
                state.env.insert(
                    param_name.to_string(),
                    DolangValue::Str(url_seg.to_string()),
                );
            } else if idx < route.params.len() {
                state.env.insert(
                    route.params[idx].clone(),
                    DolangValue::Str(url_seg.to_string()),
                );
            }
        }
    }

    if let Some(query) = &input.query {
        for pair in query.split('&') {
            let parts: Vec<&str> = pair.split('=').collect();
            if parts.len() == 2 {
                state
                    .env
                    .insert(parts[0].to_string(), DolangValue::Str(parts[1].to_string()));
            }
        }
    }

    let header_map: IndexMap<String, DolangValue> = input
        .headers
        .iter()
        .map(|(key, value)| (key.clone(), DolangValue::Str(value.clone())))
        .collect();
    state
        .env
        .insert("__headers__".to_string(), DolangValue::Json(header_map));

    if let Some(body) = &input.body {
        state.env.insert("body".to_string(), body.clone());
    }

    exec_http_handler(&route.body, &mut state, &mut runtime_context)
}

#[allow(dead_code)]
pub fn route_not_found(method: &str, path: &str) -> Error {
    Error::Interpreter(format!("Route {} {} not found", method, path))
}
