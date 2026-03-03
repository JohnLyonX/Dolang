// Int / Float 内置方法实现
use crate::error::Error;
use crate::interpreter::DolangValue;

// ===== 字符串版本（过渡期）=====

/// Format float but always keep decimal point (e.g., 42.0 not 42)
fn format_float_always(n: f64) -> String {
    if n.is_nan() {
        return "NaN".to_string();
    }
    if n.is_infinite() {
        return if n.is_sign_positive() { "inf" } else { "-inf" }.to_string();
    }

    let s = format!("{}", n);
    if !s.contains('.') {
        format!("{}.0", s)
    } else {
        s
    }
}

/// Number (Int/Float) 方法分发（字符串版本）
pub fn call_str(
    obj_val: &str,
    method: &str,
    _args: &[String],
) -> Result<String, Error> {
    let num_str = obj_val;
    let is_int = !num_str.contains('.');

    match method {
        "to_str" => {
            Ok(format!("__STR__:{}", num_str))
        }
        "to_int" => {
            if let Ok(i) = num_str.parse::<i64>() {
                Ok(i.to_string())
            } else if let Ok(f) = num_str.parse::<f64>() {
                Ok((f as i64).to_string())
            } else {
                Err(Error::Interpreter("cannot convert to Int".to_string()))
            }
        }
        "to_float" => {
            if let Ok(f) = num_str.parse::<f64>() {
                Ok(format_float_always(f))
            } else {
                Err(Error::Interpreter("cannot convert to Float".to_string()))
            }
        }
        "to_bool" => {
            if is_int {
                if let Ok(i) = num_str.parse::<i64>() {
                    if i == 0 {
                        return Ok("false".to_string());
                    } else if i == 1 {
                        return Ok("true".to_string());
                    } else {
                        return Err(Error::Interpreter(format!(
                            "cannot convert {} to Bool\nexpected 0 or 1",
                            i
                        )));
                    }
                }
            }
            Err(Error::Interpreter("type mismatch: Float cannot convert to Bool".to_string()))
        }
        "type" => {
            Ok(if is_int { "Int" } else { "Float" }.to_string())
        }
        _ => {
            Err(Error::Interpreter(format!("Number has no method '{}'", method)))
        }
    }
}

// ===== DolangValue 版本 =====

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
