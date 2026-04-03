use indexmap::IndexMap;
use uuid::Uuid;

use crate::interpreter::DolangValue;

pub const CSRF_TOKEN_CLAIM_KEY: &str = "csrf_token";

pub fn generate_csrf_token() -> String {
    format!("csrf_{}", Uuid::new_v4().simple())
}

pub fn ensure_session_csrf_token(claims: &mut IndexMap<String, DolangValue>) -> String {
    if let Some(token) = csrf_token_from_claims(claims) {
        return token;
    }

    let token = generate_csrf_token();
    claims.insert(
        CSRF_TOKEN_CLAIM_KEY.to_string(),
        DolangValue::Str(token.clone()),
    );
    token
}

pub fn replace_session_csrf_token(claims: &mut IndexMap<String, DolangValue>) -> String {
    let token = generate_csrf_token();
    claims.insert(
        CSRF_TOKEN_CLAIM_KEY.to_string(),
        DolangValue::Str(token.clone()),
    );
    token
}

pub fn csrf_token_from_claims(claims: &IndexMap<String, DolangValue>) -> Option<String> {
    match claims.get(CSRF_TOKEN_CLAIM_KEY) {
        Some(DolangValue::Str(token)) if !token.is_empty() => Some(token.clone()),
        _ => None,
    }
}
