// 内置方法分发入口
// 负责将方法调用路由到对应的类型实现

pub mod bool_methods;
pub mod file;
pub mod html;
pub mod json;
pub mod list;
pub mod map;
pub mod number;
pub mod str_methods;

use crate::error::Error;
use crate::interpreter::DolangValue;
use crate::runtime::RuntimeContext;

/// 判断方法是否需要可变访问
fn is_mutating(method: &str) -> bool {
    matches!(
        method,
        "push"
            | "pop"
            | "reverse"
            | "remove"
            | "sort"
            | "sort_desc"
            | "remove_at"
            | "insert"
            | "clear"
    )
}

/// 不可变方法分发
pub fn dispatch(
    receiver: &DolangValue,
    method: &str,
    args: &[DolangValue],
    context: &RuntimeContext,
) -> Result<DolangValue, Error> {
    match receiver {
        DolangValue::List(_) => list::call(receiver, method, args),
        DolangValue::Map(_) => map::call(receiver, method, args),
        DolangValue::Str(_) => str_methods::call(receiver, method, args),
        DolangValue::Int(_) | DolangValue::Float(_) => number::call(receiver, method, args),
        DolangValue::Bool(_) => bool_methods::call(receiver, method, args),
        DolangValue::File { .. } => file::call(receiver, method, args, context),
        DolangValue::Json(_) => json::call(receiver, method, args),
        DolangValue::Html(_) => html::call(receiver, method, args, context),
        DolangValue::Response { .. } => Err(Error::Interpreter(format!(
            "Response has no method '{}'",
            method
        ))),
        DolangValue::Function { .. } => Err(Error::Interpreter(format!(
            "Function has no method '{}'",
            method
        ))),
        DolangValue::ModuleProxy { .. } => Err(Error::Interpreter(format!(
            "ModuleProxy has no method '{}', use module.function() to call module functions",
            method
        ))),
        DolangValue::Null => Err(Error::Interpreter(format!(
            "cannot call method '{}' on null",
            method
        ))),
    }
}

/// 可变方法分发（仅 List 和 Map 需要）
pub fn dispatch_mut(
    receiver: &mut DolangValue,
    method: &str,
    args: &[DolangValue],
) -> Result<DolangValue, Error> {
    match receiver {
        DolangValue::List(_) => list::call_mut(receiver, method, args),
        DolangValue::Map(_) => map::call_mut(receiver, method, args),
        DolangValue::File { .. } => Err(Error::Interpreter(format!(
            "'{}' does not support mutable method '{}'",
            receiver.type_name(),
            method
        ))),
        DolangValue::Json(_) => Err(Error::Interpreter(format!(
            "'{}' does not support mutable method '{}'",
            receiver.type_name(),
            method
        ))),
        DolangValue::Response { .. } => Err(Error::Interpreter(format!(
            "'{}' does not support mutable method '{}'",
            receiver.type_name(),
            method
        ))),
        _ => Err(Error::Interpreter(format!(
            "'{}' does not support mutable method '{}'",
            receiver.type_name(),
            method
        ))),
    }
}

/// 判断方法是否为可变方法（会修改原对象）
pub fn is_method_mutating(method: &str) -> bool {
    is_mutating(method)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::runtime::{RuntimeContext, RuntimeMode};

    #[test]
    fn dispatches_string_and_list_methods() {
        let context = RuntimeContext::new(RuntimeMode::Test, PathBuf::from("."));
        let upper = dispatch(
            &DolangValue::Str("dolang".to_string()),
            "upper",
            &[],
            &context,
        )
        .expect("upper should work");
        let len = dispatch(
            &DolangValue::List(vec![DolangValue::Int(1), DolangValue::Int(2)]),
            "len",
            &[],
            &context,
        )
        .expect("len should work");

        assert_eq!(upper, DolangValue::Str("DOLANG".to_string()));
        assert_eq!(len, DolangValue::Int(2));
    }
}
