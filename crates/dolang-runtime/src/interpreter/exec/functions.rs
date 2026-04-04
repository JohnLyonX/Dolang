use crate::ast::FnDeclStmt;
use crate::error::Error;
use crate::interpreter::{TypeValidationError, validate_value_against_type};
use crate::runtime::{ProgramState, RuntimeContext};

use super::super::env::{FnEnv, RuntimeFn};
use super::super::value::DolangValue;
use super::{Flow, exec_block};

pub(super) fn handle_fn_decl(stmt: &FnDeclStmt, state: &mut ProgramState) -> Flow {
    if state.lookup_fn(&stmt.name).is_some() {
        return Flow::Err(Error::Interpreter(format!(
            "function '{}' is already defined",
            stmt.name
        )));
    }
    let module_env = state.capture_env_snapshot();
    state.fns.insert(
        stmt.name.clone(),
        RuntimeFn {
            decl: stmt.clone(),
            source_file: None,
            module_env,
        },
    );
    Flow::Normal
}

pub fn call_fn(
    fn_def: &RuntimeFn,
    args: &[DolangValue],
    fns: &mut FnEnv,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Result<Option<DolangValue>, Error> {
    let fn_decl = &fn_def.decl;
    let min_params = fn_decl.params.len();
    let has_variadic = fn_decl.variadic_param.is_some();
    if has_variadic && args.len() < min_params {
        return Err(Error::Interpreter(format!(
            "function '{}' expects at least {} arguments, got {}",
            fn_decl.name,
            min_params,
            args.len()
        )));
    }
    if !has_variadic && args.len() != min_params {
        return Err(Error::Interpreter(format!(
            "function '{}' expects {} arguments, got {}",
            fn_decl.name,
            min_params,
            args.len()
        )));
    }

    let mut local_state = ProgramState::new();
    local_state.set_env_fallback(fn_def.module_env.clone());

    for (param, arg) in fn_decl.params.iter().zip(args.iter()) {
        local_state.insert_env(param.clone(), arg.clone());
    }

    if let Some(var_param) = &fn_decl.variadic_param {
        let extra_args: Vec<DolangValue> = args[min_params..].to_vec();
        local_state.insert_env(var_param.clone(), DolangValue::List(extra_args));
    }

    local_state.fns = fns.clone();

    let flow = exec_block(&fn_decl.body, &mut local_state, context, w);
    *fns = local_state.fns.clone();
    match flow {
        Flow::Return(val) => {
            validate_declared_return_type(
                "function",
                &fn_decl.name,
                fn_decl.return_type.as_deref(),
                val.as_ref(),
                context,
            )?;
            Ok(val)
        }
        Flow::Normal => Ok(None),
        Flow::Err(err) => Err(err),
        Flow::Exit => Ok(None),
        Flow::Throw(val) => Err(Error::Interpreter(format!("uncaught throw: {val}"))),
        Flow::Break | Flow::Continue => Err(Error::Interpreter(
            "break/continue used outside of loop".to_string(),
        )),
    }
}

/// 调用模块内的 Dolang 函数，预注入模块执行时的环境（已导入的模块代理、常量等）。
pub fn call_module_fn(
    fn_def: &RuntimeFn,
    args: &[DolangValue],
    caller_fns: &mut FnEnv,
    module_fns: &FnEnv,
    module_env: &crate::interpreter::env::Env,
    context: &mut RuntimeContext,
    w: &mut dyn std::io::Write,
) -> Result<Option<DolangValue>, Error> {
    let fn_decl = &fn_def.decl;
    let min_params = fn_decl.params.len();
    let has_variadic = fn_decl.variadic_param.is_some();
    if has_variadic && args.len() < min_params {
        return Err(Error::Interpreter(format!(
            "function '{}' expects at least {} arguments, got {}",
            fn_decl.name,
            min_params,
            args.len()
        )));
    }
    if !has_variadic && args.len() != min_params {
        return Err(Error::Interpreter(format!(
            "function '{}' expects {} arguments, got {}",
            fn_decl.name,
            min_params,
            args.len()
        )));
    }

    let mut local_state = ProgramState::new();

    // 模块环境作为只读 fallback，本地绑定按需覆写。
    local_state.set_env_fallback(std::sync::Arc::new(module_env.clone()));

    // 参数绑定（覆盖同名的模块环境变量）
    for (param, arg) in fn_decl.params.iter().zip(args.iter()) {
        local_state.insert_env(param.clone(), arg.clone());
    }
    if let Some(var_param) = &fn_decl.variadic_param {
        let extra = args[min_params..].to_vec();
        local_state.insert_env(var_param.clone(), DolangValue::List(extra));
    }

    local_state.fns = module_fns.clone();
    local_state.set_fn_fallback(caller_fns.clone());

    let flow = exec_block(&fn_decl.body, &mut local_state, context, w);

    match flow {
        Flow::Return(val) => {
            validate_declared_return_type(
                "function",
                &fn_decl.name,
                fn_decl.return_type.as_deref(),
                val.as_ref(),
                context,
            )?;
            Ok(val)
        }
        Flow::Normal => Ok(None),
        Flow::Err(err) => Err(err),
        Flow::Exit => Ok(None),
        Flow::Throw(val) => Err(Error::Interpreter(format!("uncaught throw: {val}"))),
        Flow::Break | Flow::Continue => Err(Error::Interpreter(
            "break/continue used outside of loop".to_string(),
        )),
    }
}

pub fn validate_declared_return_type(
    kind: &str,
    name: &str,
    expected_type: Option<&str>,
    value: Option<&DolangValue>,
    context: &RuntimeContext,
) -> Result<(), Error> {
    let Some(expected_type) = expected_type else {
        return Ok(());
    };

    let Some(value) = value else {
        return Err(Error::Interpreter(format!(
            "{kind} '{name}' expects return type '{expected_type}' but returned nothing"
        )));
    };

    match validate_value_against_type(expected_type, value, context) {
        Ok(()) => Ok(()),
        Err(TypeValidationError::BareList) => Err(Error::Interpreter(format!(
            "{kind} '{name}' declares return type 'List' without a type parameter; use 'List<T>' instead (e.g. 'List<User>')"
        ))),
        Err(TypeValidationError::UnknownType { expected_type }) => Err(Error::Interpreter(
            format!("{kind} '{name}' references unknown return type '{expected_type}'"),
        )),
        Err(TypeValidationError::Mismatch {
            expected_type,
            actual_type,
        }) => Err(Error::Interpreter(format!(
            "{kind} '{name}' expects return type '{expected_type}' but got '{actual_type}'"
        ))),
        Err(TypeValidationError::MissingRequiredField { .. })
        | Err(TypeValidationError::UnknownField { .. }) => Err(Error::Interpreter(format!(
            "{kind} '{name}' expects return type '{expected_type}' but got '{}'",
            value.type_name()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{
        RuntimeMode,
        context::{TypeField, TypeShape},
    };
    use indexmap::IndexMap;
    use std::path::PathBuf;

    fn test_context() -> RuntimeContext {
        RuntimeContext::new(RuntimeMode::Test, PathBuf::from("."))
    }

    #[test]
    fn shared_return_type_validator_accepts_matching_int() {
        let context = test_context();
        let value = DolangValue::Int(1);

        assert!(
            validate_declared_return_type("function", "f", Some("Int"), Some(&value), &context,)
                .is_ok()
        );
    }

    #[test]
    fn shared_return_type_validator_rejects_missing_return_value() {
        let context = test_context();
        let error = validate_declared_return_type("function", "f", Some("Int"), None, &context)
            .expect_err("missing return should fail");
        assert!(error.to_string().contains("returned nothing"));
    }

    #[test]
    fn shared_return_type_validator_rejects_wrong_user_type() {
        let mut context = test_context();
        context.register_type(
            "User",
            TypeShape {
                name: "User".to_string(),
                fields: vec![TypeField {
                    name: "id".to_string(),
                    type_name: "Int".to_string(),
                    optional: false,
                    hidden: false,
                }],
            },
        );

        let value = DolangValue::Map(IndexMap::new());

        let error =
            validate_declared_return_type("function", "f", Some("User"), Some(&value), &context)
                .expect_err("wrong user type should fail");
        assert!(error.to_string().contains("expects return type 'User'"));
    }

    #[test]
    fn shared_return_type_validator_accepts_list_of_user_instances() {
        let mut context = test_context();
        context.register_type(
            "User",
            TypeShape {
                name: "User".to_string(),
                fields: vec![TypeField {
                    name: "id".to_string(),
                    type_name: "Int".to_string(),
                    optional: false,
                    hidden: false,
                }],
            },
        );

        let value = DolangValue::List(vec![DolangValue::TypedInstance {
            type_name: "User".to_string(),
            fields: IndexMap::new(),
        }]);

        assert!(
            validate_declared_return_type(
                "function",
                "f",
                Some("List<User>"),
                Some(&value),
                &context,
            )
            .is_ok()
        );
    }
}
