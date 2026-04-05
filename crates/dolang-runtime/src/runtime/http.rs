use chrono::Utc;
use indexmap::IndexMap;
use uuid::Uuid;

use crate::ast::FnParam;
use crate::error::Error;
use crate::interpreter::exec::{parameter_type_error, validate_declared_return_type};
use crate::interpreter::{DolangValue, HttpRoute, exec_http_handler, validate_value_against_type_expr};
use crate::runtime::auth::{
    Principal, RuntimeAuthorizationRule, SessionStore, SessionStoreBackend, csrf_token_from_claims,
    ensure_session_csrf_token, verify_jwt,
};

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
    let mut runtime_context = context.clone_for_request_execution();
    execute_http_route_in_context(route, input, &mut runtime_context)
}

pub fn execute_http_route_in_context(
    route: &HttpRoute,
    input: &HandlerInput,
    runtime_context: &mut RuntimeContext,
) -> (bool, Option<DolangValue>, Option<String>) {
    let mut state = ProgramState::new();

    if let Err(error) = apply_request_auth(route, input, runtime_context) {
        return (true, None, Some(error));
    }

    // Seed module-level imports ($mod) and functions ($fn) captured when
    // the route's source file was loaded via $HTTP.link().
    state.extend_env(route.module_env().clone());
    state.fns.extend(route.module_fns().clone());

    let route_seg_parts: Vec<&str> = route.path.split('/').collect();
    let url_seg_parts: Vec<&str> = input.request_path.split('/').collect();

    let mut bound_http_params: IndexMap<String, DolangValue> = IndexMap::new();

    if route_seg_parts.len() == url_seg_parts.len() {
        for idx in 0..route_seg_parts.len() {
            let route_seg = route_seg_parts[idx];
            let url_seg = url_seg_parts[idx];
            if let Some(param_name) = route_seg.strip_prefix(':') {
                bound_http_params.insert(
                    param_name.to_string(),
                    DolangValue::Str(url_seg.to_string()),
                );
            } else if idx < route.params.len() {
                bound_http_params.insert(
                    route.params[idx].name.clone(),
                    DolangValue::Str(url_seg.to_string()),
                );
            }
        }
    }

    if let Some(query) = &input.query {
        for pair in query.split('&') {
            let parts: Vec<&str> = pair.split('=').collect();
            if parts.len() == 2 {
                bound_http_params.insert(
                    parts[0].to_string(),
                    DolangValue::Str(parts[1].to_string()),
                );
            }
        }
    }

    if let Err(err) =
        bind_http_handler_params(&route.params, &bound_http_params, &mut state, runtime_context)
    {
        return (true, None, Some(err.to_string()));
    }
    for (name, value) in &bound_http_params {
        if !state.env_contains_key(name) {
            state.insert_env(name.clone(), value.clone());
        }
    }

    let header_map: IndexMap<String, DolangValue> = input
        .headers
        .iter()
        .map(|(key, value)| (key.clone(), DolangValue::Str(value.clone())))
        .collect();
    state.insert_env("__headers__".to_string(), DolangValue::Json(header_map));

    if let Some(body) = &input.body {
        state.insert_env("body".to_string(), body.clone());
    }

    let (should_continue, result, error) =
        exec_http_handler(&route.body, &mut state, runtime_context);

    if error.is_none() {
        if let Err(e) = validate_declared_return_type(
            "http handler",
            &route.name,
            route.return_type.as_ref(),
            result.as_ref(),
            runtime_context,
        ) {
            return (true, None, Some(e.to_string()));
        }
    }

    (should_continue, result, error)
}

pub(crate) fn bind_http_handler_params(
    params: &[FnParam],
    values: &IndexMap<String, DolangValue>,
    state: &mut ProgramState,
    runtime_context: &RuntimeContext,
) -> Result<(), Error> {
    for param in params {
        if let Some(value) = values.get(&param.name) {
            if let Some(type_expr) = &param.type_annotation {
                validate_value_against_type_expr(type_expr, value, runtime_context)
                    .map_err(|error| parameter_type_error(&param.name, type_expr, error))?;
            }
            state.insert_env(param.name.clone(), value.clone());
        }
    }

    Ok(())
}

fn apply_request_auth(
    route: &HttpRoute,
    input: &HandlerInput,
    context: &RuntimeContext,
) -> Result<(), String> {
    let auth_config = context.runtime_auth_config();
    if !auth_config.enabled {
        return Ok(());
    }

    for source in &auth_config.identity_sources {
        if context
            .with_request_auth_context(|auth| auth.principal().is_some())
            .map_err(|err| err.to_string())?
        {
            break;
        }

        match source.as_str() {
            "cookie" => {
                if let Some(session_id) =
                    extract_cookie_value(input, &auth_config.session_cookie_name)
                {
                    apply_session_auth(context, &session_id)?;
                }
            }
            "bearer" => {
                if let Some(token) = extract_bearer_token(input) {
                    apply_bearer_auth(context, &token)?;
                }
            }
            _ => {}
        }
    }

    let principal = context
        .with_request_auth_context(|auth| auth.principal().cloned())
        .map_err(|err| err.to_string())?;
    let rule = auth_config.matching_rule(&route.method, &route.path);

    if auth_config.requires_authentication(&route.method, &route.path) && principal.is_none() {
        return Err("__auth_unauthorized__".to_string());
    }

    if let Some(principal) = principal
        && let Some(rule) = rule
        && !principal_satisfies_rule(&principal, rule)
    {
        return Err("__auth_forbidden__".to_string());
    }

    enforce_csrf_if_needed(route, input, context)?;

    Ok(())
}

fn apply_session_auth(context: &RuntimeContext, session_id: &str) -> Result<(), String> {
    let auth_config = context.runtime_auth_config();
    if !auth_config.session_enabled {
        return Ok(());
    }

    let session = context
        .with_session_store_mut(|store: &mut SessionStoreBackend| store.get(session_id))
        .map_err(|err| err.to_string())?;
    let Some(mut session) = session else {
        return Ok(());
    };
    let had_csrf_token = csrf_token_from_claims(&session.claims).is_some();
    ensure_session_csrf_token(&mut session.claims);

    let now = Utc::now().timestamp();
    let expired = session.expires_at <= now
        || session
            .idle_timeout_at
            .map(|idle_timeout_at| idle_timeout_at <= now)
            .unwrap_or(false);
    if expired {
        context
            .with_session_store_mut(|store: &mut SessionStoreBackend| {
                store.delete(&session.session_id)
            })
            .map_err(|err| err.to_string())?;
        context
            .with_request_auth_context_mut(|auth| {
                auth.set_current_session(None);
                auth.set_principal(None);
                auth.set_pending_cookie(None);
                auth.set_pending_clear_cookie(true);
            })
            .map_err(|err| err.to_string())?;
        return Ok(());
    }

    if auth_config.session_idle_timeout_seconds > 0 {
        session.idle_timeout_at = Some(now + auth_config.session_idle_timeout_seconds);
    }

    if auth_config.session_rotation == "always" {
        let old_session_id = session.session_id.clone();
        session.session_id = format!("sess_{}", Uuid::new_v4().simple());
        context
            .with_request_auth_context_mut(|auth| {
                auth.queue_session_rotate(old_session_id, session.clone());
                auth.set_pending_cookie(Some(
                    context
                        .runtime_auth_config()
                        .cookie_header_value(&session.session_id),
                ));
                auth.set_pending_clear_cookie(false);
            })
            .map_err(|err| err.to_string())?;
    } else if auth_config.session_idle_timeout_seconds > 0 || !had_csrf_token {
        context
            .with_request_auth_context_mut(|auth| {
                auth.queue_session_update(session.clone());
            })
            .map_err(|err| err.to_string())?;
    }

    let principal = Principal {
        subject: session.subject.clone(),
        scheme: "session".to_string(),
        roles: session.roles.clone(),
        permissions: session.permissions.clone(),
        claims: session.claims.clone(),
        session_id: Some(session.session_id.clone()),
    };
    context
        .with_request_auth_context_mut(|auth| {
            auth.set_current_session(Some(session));
            auth.set_principal(Some(principal));
        })
        .map_err(|err| err.to_string())
}

fn enforce_csrf_if_needed(
    route: &HttpRoute,
    input: &HandlerInput,
    context: &RuntimeContext,
) -> Result<(), String> {
    if !requires_csrf_protection(&route.method) {
        return Ok(());
    }

    let (principal_scheme, expected_token) = context
        .with_request_auth_context(|auth| {
            (
                auth.principal().map(|principal| principal.scheme.clone()),
                auth.current_session()
                    .and_then(|session| csrf_token_from_claims(&session.claims)),
            )
        })
        .map_err(|err| err.to_string())?;

    if principal_scheme.as_deref() != Some("session") {
        return Ok(());
    }

    let Some(expected_token) = expected_token else {
        return Err("__auth_forbidden__: invalid csrf token".to_string());
    };
    let provided_token = input
        .headers
        .get("x-csrf-token")
        .or_else(|| input.headers.get("X-CSRF-Token"));

    if provided_token.is_some_and(|token| token == &expected_token) {
        Ok(())
    } else {
        Err("__auth_forbidden__: invalid csrf token".to_string())
    }
}

fn requires_csrf_protection(method: &str) -> bool {
    matches!(method, "POST" | "PUT" | "PATCH" | "DELETE")
}

fn apply_bearer_auth(context: &RuntimeContext, token: &str) -> Result<(), String> {
    let auth_config = context.runtime_auth_config();
    if !auth_config.jwt_enabled {
        return Ok(());
    }

    let claims = match verify_jwt(auth_config, token) {
        Ok(claims) => claims,
        Err(_) => return Ok(()),
    };

    let principal_claims: IndexMap<String, DolangValue> = claims
        .extra
        .iter()
        .map(|(key, value)| (key.clone(), json_value_to_dolang(value.clone())))
        .collect();
    let current_claims = jwt_claims_to_map(&claims);
    let principal = Principal {
        subject: claims.sub.clone(),
        scheme: "bearer".to_string(),
        roles: claims.roles.clone(),
        permissions: claims.permissions.clone(),
        claims: principal_claims,
        session_id: None,
    };
    context
        .with_request_auth_context_mut(|auth| {
            auth.set_principal(Some(principal));
            auth.set_current_jwt_claims(Some(current_claims));
            auth.set_current_bearer_token(Some(token.to_string()));
        })
        .map_err(|err| err.to_string())
}

fn principal_satisfies_rule(principal: &Principal, rule: &RuntimeAuthorizationRule) -> bool {
    if !rule.roles_any.is_empty()
        && !rule
            .roles_any
            .iter()
            .any(|role| principal.roles.iter().any(|owned| owned == role))
    {
        return false;
    }

    if !rule.roles_all.is_empty()
        && !rule
            .roles_all
            .iter()
            .all(|role| principal.roles.iter().any(|owned| owned == role))
    {
        return false;
    }

    if !rule.permissions_any.is_empty()
        && !rule.permissions_any.iter().any(|permission| {
            principal
                .permissions
                .iter()
                .any(|owned| owned == permission)
        })
    {
        return false;
    }

    if !rule.permissions_all.is_empty()
        && !rule.permissions_all.iter().all(|permission| {
            principal
                .permissions
                .iter()
                .any(|owned| owned == permission)
        })
    {
        return false;
    }

    if !rule.claims_all.is_empty()
        && !rule.claims_all.iter().all(|(key, expected)| {
            principal
                .claims
                .get(key)
                .map(claim_value_to_string)
                .is_some_and(|actual| actual == *expected)
        })
    {
        return false;
    }

    true
}

fn claim_value_to_string(value: &DolangValue) -> String {
    match value {
        DolangValue::Str(value) => value.clone(),
        DolangValue::Int(value) => value.to_string(),
        DolangValue::Float(value) => value.to_string(),
        DolangValue::Bool(value) => value.to_string(),
        DolangValue::Null => "null".to_string(),
        other => other.to_string(),
    }
}

fn extract_bearer_token(input: &HandlerInput) -> Option<String> {
    let header = input
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("authorization"))
        .map(|(_, value)| value)?;
    header.strip_prefix("Bearer ").map(str::to_string)
}

fn extract_cookie_value(input: &HandlerInput, cookie_name: &str) -> Option<String> {
    let cookie_header = input
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("cookie"))
        .map(|(_, value)| value)?;

    for pair in cookie_header.split(';') {
        let trimmed = pair.trim();
        if let Some(value) = trimmed.strip_prefix(&format!("{cookie_name}=")) {
            return Some(value.to_string());
        }
    }

    None
}

fn jwt_claims_to_map(claims: &crate::runtime::auth::JwtClaims) -> IndexMap<String, DolangValue> {
    let mut map = IndexMap::new();
    map.insert("sub".to_string(), DolangValue::Str(claims.sub.clone()));
    map.insert("iss".to_string(), DolangValue::Str(claims.iss.clone()));
    map.insert("aud".to_string(), DolangValue::Str(claims.aud.clone()));
    map.insert("exp".to_string(), DolangValue::Int(claims.exp as i64));
    map.insert(
        "token_use".to_string(),
        DolangValue::Str(claims.token_use.clone()),
    );
    map.insert(
        "roles".to_string(),
        DolangValue::List(
            claims
                .roles
                .iter()
                .map(|value| DolangValue::Str(value.clone()))
                .collect(),
        ),
    );
    map.insert(
        "permissions".to_string(),
        DolangValue::List(
            claims
                .permissions
                .iter()
                .map(|value| DolangValue::Str(value.clone()))
                .collect(),
        ),
    );
    for (key, value) in &claims.extra {
        map.insert(key.clone(), json_value_to_dolang(value.clone()));
    }
    map
}

fn json_value_to_dolang(value: serde_json::Value) -> DolangValue {
    match value {
        serde_json::Value::Null => DolangValue::Null,
        serde_json::Value::Bool(b) => DolangValue::Bool(b),
        serde_json::Value::Number(n) => value_to_json_number(&n),
        serde_json::Value::String(s) => DolangValue::Str(s),
        serde_json::Value::Array(values) => {
            DolangValue::List(values.into_iter().map(json_value_to_dolang).collect())
        }
        serde_json::Value::Object(entries) => DolangValue::Map(
            entries
                .into_iter()
                .map(|(key, value)| (key, json_value_to_dolang(value)))
                .collect(),
        ),
    }
}

fn value_to_json_number(number: &serde_json::Number) -> DolangValue {
    if let Some(int) = number.as_i64() {
        DolangValue::Int(int)
    } else if let Some(float) = number.as_f64() {
        DolangValue::Float(float)
    } else {
        DolangValue::Null
    }
}

#[allow(dead_code)]
pub fn route_not_found(method: &str, path: &str) -> Error {
    Error::Interpreter(format!("Route {} {} not found", method, path))
}
