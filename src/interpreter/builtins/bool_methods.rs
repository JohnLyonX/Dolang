// Bool 内置方法实现
use crate::error::Error;
use crate::interpreter::DolangValue;

// ===== 字符串版本（过渡期）=====

/// Bool 方法分发（字符串版本）
pub fn call_str(
    obj_val: &str,
    method: &str,
    _args: &[String],
) -> Result<String, Error> {
    let b = obj_val == "true";

    match method {
        "to_str" => Ok(format!("__STR__:{}", b)),
        "to_int" => Err(Error::Interpreter("type mismatch: Bool cannot convert to Int".to_string())),
        "to_float" => Err(Error::Interpreter("type mismatch: Bool cannot convert to Float".to_string())),
        "to_bool" => Ok(b.to_string()),
        "type" => Ok("Bool".to_string()),
        _ => Err(Error::Interpreter(format!("Bool has no method '{}'", method))),
    }
}

// ===== DolangValue 版本 =====

/// Bool 方法分发
pub fn call(
    receiver: &DolangValue,
    method: &str,
    _args: &[DolangValue],
) -> Result<DolangValue, Error> {
    let b = match receiver {
        DolangValue::Bool(b) => *b,
        _ => return Err(Error::Interpreter("expected Bool".to_string())),
    };

    match method {
        "to_str" => Ok(DolangValue::Str(b.to_string())),
        "to_int" => Err(Error::Interpreter("type mismatch: Bool cannot convert to Int".to_string())),
        "to_float" => Err(Error::Interpreter("type mismatch: Bool cannot convert to Float".to_string())),
        "to_bool" => Ok(DolangValue::Bool(b)),
        "type" => Ok(DolangValue::Str("Bool".to_string())),
        _ => Err(Error::Interpreter(format!("Bool has no method '{}'", method))),
    }
}
