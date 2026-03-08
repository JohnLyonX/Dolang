// 内置方法分发入口
// 负责将方法调用路由到对应的类型实现

pub mod list;
pub mod map;
pub mod number;
pub mod str_methods;
pub mod bool_methods;
pub mod file;
pub mod html;

use crate::error::Error;
use crate::interpreter::DolangValue;

/// 判断方法是否需要可变访问
fn is_mutating(method: &str) -> bool {
    matches!(method, "push" | "pop" | "reverse" | "remove")
}

/// 不可变方法分发
pub fn dispatch(
    receiver: &DolangValue,
    method: &str,
    args: &[DolangValue],
) -> Result<DolangValue, Error> {
    match receiver {
        DolangValue::List(_) => list::call(receiver, method, args),
        DolangValue::Map(_) => map::call(receiver, method, args),
        DolangValue::Str(_) => str_methods::call(receiver, method, args),
        DolangValue::Int(_) | DolangValue::Float(_) => number::call(receiver, method, args),
        DolangValue::Bool(_) => bool_methods::call(receiver, method, args),
        DolangValue::File { .. } => file::call(receiver, method, args),
        DolangValue::Json(_) => Err(Error::Interpreter(format!(
            "Json has no method '{}'",
            method
        ))),
        DolangValue::Html(_) => html::call(receiver, method, args),
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
