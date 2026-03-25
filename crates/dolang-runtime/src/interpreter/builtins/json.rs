// JSON 内置方法实现

use crate::error::Error;
use crate::interpreter::DolangValue;

/// JSON 方法分发（不可变）
pub fn call(
    receiver: &DolangValue,
    method: &str,
    _args: &[DolangValue],
) -> Result<DolangValue, Error> {
    match method {
        "to_str" => {
            // 将 Json 转换为字符串表示
            Ok(DolangValue::Str(receiver.to_string()))
        }
        "type" => Ok(DolangValue::Str("Json".to_string())),
        _ => Err(Error::Interpreter(format!(
            "Json has no method '{}'",
            method
        ))),
    }
}
