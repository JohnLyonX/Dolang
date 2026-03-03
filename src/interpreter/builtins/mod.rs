// 内置方法分发入口
// 负责将方法调用路由到对应的类型实现

pub mod list;
pub mod map;
pub mod number;
pub mod str_methods;
pub mod bool_methods;

use crate::error::Error;
use crate::interpreter::DolangValue;

// ===== 过渡期兼容层 =====

/// 临时兼容函数：eval.rs 还在使用字符串版本
/// 过渡期结束后删除
pub fn dispatch_str(
    obj_val: &str,
    method: &str,
    args: &[String],
) -> Result<String, Error> {
    // 解析为 DolangValue
    let value = match DolangValue::parse_legacy(obj_val) {
        Some(v) => v,
        None => return Err(Error::Interpreter(format!("failed to parse value: {}", obj_val))),
    };

    // 转换参数
    let args_values: Vec<DolangValue> = args
        .iter()
        .filter_map(|s| DolangValue::parse_legacy(s))
        .collect();

    // 调用 DolangValue 版本
    let result = dispatch(&value, method, &args_values)?;

    // 转回字符串
    Ok(result.to_legacy())
}

/// 临时兼容函数：eval.rs 还在使用字符串版本
pub fn dispatch_mut_str(
    obj_val: &str,
    method: &str,
    args: &[String],
) -> Result<String, Error> {
    // 解析为 DolangValue
    let mut value = match DolangValue::parse_legacy(obj_val) {
        Some(v) => v,
        None => return Err(Error::Interpreter(format!("failed to parse value: {}", obj_val))),
    };

    // 转换参数
    let args_values: Vec<DolangValue> = args
        .iter()
        .filter_map(|s| DolangValue::parse_legacy(s))
        .collect();

    // 调用可变方法版本
    let result = dispatch_mut(&mut value, method, &args_values)?;

    // 转回字符串
    Ok(result.to_legacy())
}

// ===== 正式版本 =====

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
        DolangValue::Function { .. } => Err(Error::Interpreter(format!(
            "Function has no method '{}'",
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
