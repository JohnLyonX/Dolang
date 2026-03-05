// Int / Float 内置方法实现
use crate::error::Error;
use crate::interpreter::DolangValue;

/// Number (Int/Float) 方法分发
pub fn call(
    receiver: &DolangValue,
    method: &str,
    _args: &[DolangValue],
) -> Result<DolangValue, Error> {
    match receiver {
        DolangValue::Int(n) => call_int(*n, method),
        DolangValue::Float(f) => call_float(*f, method),
        _ => Err(Error::Interpreter("expected Int or Float".to_string())),
    }
}

fn call_int(n: i64, method: &str) -> Result<DolangValue, Error> {
    match method {
        "to_str" => Ok(DolangValue::Str(n.to_string())),
        "to_int" => Ok(DolangValue::Int(n)),
        "to_float" => Ok(DolangValue::Float(n as f64)),
        "to_bool" => {
            if n == 0 {
                Ok(DolangValue::Bool(false))
            } else if n == 1 {
                Ok(DolangValue::Bool(true))
            } else {
                Err(Error::Interpreter(format!(
                    "cannot convert {} to Bool\nexpected 0 or 1",
                    n
                )))
            }
        }
        "type" => Ok(DolangValue::Str("Int".to_string())),
        _ => Err(Error::Interpreter(format!("Int has no method '{}'", method))),
    }
}

fn call_float(f: f64, method: &str) -> Result<DolangValue, Error> {
    match method {
        "to_str" => {
            let s = if f.fract().abs() < 1e-10 {
                format!("{:.1}", f)
            } else {
                f.to_string()
            };
            Ok(DolangValue::Str(s))
        }
        "to_int" => Ok(DolangValue::Int(f as i64)),
        "to_float" => Ok(DolangValue::Float(f)),
        "to_bool" => Err(Error::Interpreter("type mismatch: Float cannot convert to Bool".to_string())),
        "type" => Ok(DolangValue::Str("Float".to_string())),
        _ => Err(Error::Interpreter(format!("Float has no method '{}'", method))),
    }
}
