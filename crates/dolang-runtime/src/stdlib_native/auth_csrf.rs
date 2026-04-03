use std::sync::Arc;

use crate::error::Error;
use crate::interpreter::DolangValue;
use crate::runtime::auth::{csrf_token_from_claims, replace_session_csrf_token};
use crate::runtime::{NativeFnMap, RuntimeContext};

pub fn register(context: &mut RuntimeContext) {
    let mut exports = NativeFnMap::new();
    exports.insert(
        "token".into(),
        Arc::new(|_args: &[DolangValue], ctx: &RuntimeContext| {
            let token = ctx.with_request_auth_context(|auth| {
                auth.current_session()
                    .and_then(|session| csrf_token_from_claims(&session.claims))
            })?;
            Ok(token.map(DolangValue::Str).unwrap_or(DolangValue::Null))
        }),
    );
    exports.insert(
        "rotate".into(),
        Arc::new(|_args: &[DolangValue], ctx: &RuntimeContext| {
            let token = ctx.with_request_auth_context_mut(|auth| {
                let mut session = auth.current_session().cloned().ok_or_else(|| {
                    Error::Interpreter("auth.csrf.rotate: no active session".to_string())
                })?;
                let token = replace_session_csrf_token(&mut session.claims);
                auth.set_current_session(Some(session.clone()));
                if let Some(principal) = auth.principal().cloned() {
                    let mut principal = principal;
                    principal.claims = session.claims.clone();
                    auth.set_principal(Some(principal));
                }
                auth.queue_session_update(session);
                Ok::<String, Error>(token)
            })??;
            Ok(DolangValue::Str(token))
        }),
    );
    context.register_native_module("std.auth.csrf", exports);
}
