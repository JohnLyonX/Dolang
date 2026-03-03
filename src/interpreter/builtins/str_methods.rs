// String 内置方法实现
use crate::error::Error;
use crate::interpreter::DolangValue;
use crate::interpreter::env::serialize_list;

// ===== 字符串版本（过渡期）=====

/// String 方法分发（字符串版本）
pub fn call_str(
    obj_val: &str,
    method: &str,
    args: &[String],
) -> Result<String, Error> {
    let s = if obj_val.starts_with("__STR__:") {
        obj_val.trim_start_matches("__STR__:")
    } else {
        obj_val
    };

    match method {
        "len" | "length" => Ok(s.len().to_string()),
        "upper" => Ok(s.to_uppercase()),
        "lower" => Ok(s.to_lowercase()),
        "trim" => Ok(s.trim().to_string()),
        "contains" => {
            if args.is_empty() {
                return Err(Error::Interpreter("method 'contains' requires 1 argument".to_string()));
            }
            let arg = if args[0].starts_with("__STR__:") {
                args[0].trim_start_matches("__STR__:")
            } else {
                &args[0]
            };
            Ok(s.contains(arg).to_string())
        }
        "starts_with" => {
            if args.is_empty() {
                return Err(Error::Interpreter("method 'starts_with' requires 1 argument".to_string()));
            }
            let arg = if args[0].starts_with("__STR__:") {
                args[0].trim_start_matches("__STR__:")
            } else {
                &args[0]
            };
            Ok(s.starts_with(arg).to_string())
        }
        "ends_with" => {
            if args.is_empty() {
                return Err(Error::Interpreter("method 'ends_with' requires 1 argument".to_string()));
            }
            let arg = if args[0].starts_with("__STR__:") {
                args[0].trim_start_matches("__STR__:")
            } else {
                &args[0]
            };
            Ok(s.ends_with(arg).to_string())
        }
        "replace" => {
            if args.len() != 2 {
                return Err(Error::Interpreter("method 'replace' requires 2 arguments".to_string()));
            }
            let old = if args[0].starts_with("__STR__:") {
                args[0].trim_start_matches("__STR__:")
            } else {
                &args[0]
            };
            let new = if args[1].starts_with("__STR__:") {
                args[1].trim_start_matches("__STR__:")
            } else {
                &args[1]
            };
            Ok(s.replace(old, new))
        }
        "split" => {
            if args.is_empty() {
                return Err(Error::Interpreter("method 'split' requires 1 argument".to_string()));
            }
            let sep = if args[0].starts_with("__STR__:") {
                args[0].trim_start_matches("__STR__:")
            } else {
                &args[0]
            };
            let parts: Vec<String> = s.split(sep).map(|s| format!("__STR__:{}", s)).collect();
            Ok(serialize_list(&parts))
        }
        "slice" => {
            if args.len() != 2 {
                return Err(Error::Interpreter("method 'slice' requires 2 arguments".to_string()));
            }
            if let (Ok(start), Ok(end)) = (args[0].parse::<usize>(), args[1].parse::<usize>()) {
                let chars: Vec<char> = s.chars().collect();
                if start < chars.len() && end <= chars.len() && start < end {
                    let slice: String = chars[start..end].iter().collect();
                    return Ok(format!("__STR__:{}", slice));
                }
            }
            Err(Error::Interpreter("invalid slice indices".to_string()))
        }
        "to_int" => {
            if let Ok(i) = s.parse::<i64>() {
                Ok(i.to_string())
            } else if let Ok(f) = s.parse::<f64>() {
                Ok((f as i64).to_string())
            } else {
                Err(Error::Interpreter(format!("cannot convert \"{}\" to Int", s)))
            }
        }
        "to_float" => {
            if let Ok(f) = s.parse::<f64>() {
                let result = if f.fract().abs() < 1e-10 {
                    format!("{:.1}", f)
                } else {
                    f.to_string()
                };
                Ok(result)
            } else {
                Err(Error::Interpreter(format!("cannot convert \"{}\" to Float", s)))
            }
        }
        "to_bool" => {
            if s == "true" {
                Ok("true".to_string())
            } else if s == "false" {
                Ok("false".to_string())
            } else {
                Err(Error::Interpreter(format!(
                    "cannot convert \"{}\" to Bool\nexpected \"true\" or \"false\"",
                    s
                )))
            }
        }
        "to_str" => {
            if obj_val.starts_with("__STR__:") {
                Ok(s.to_string())
            } else {
                Ok(obj_val.to_string())
            }
        }
        "type" => Ok("String".to_string()),
        _ => Err(Error::Interpreter(format!("string has no method '{}'", method))),
    }
}

// ===== DolangValue 版本 =====

/// String 方法分发
pub fn call(
    receiver: &DolangValue,
    method: &str,
    args: &[DolangValue],
) -> Result<DolangValue, Error> {
    let s = match receiver {
        DolangValue::Str(s) => s,
        _ => return Err(Error::Interpreter("expected String".to_string())),
    };

    match method {
        "len" | "length" => Ok(DolangValue::Int(s.len() as i64)),
        "upper" => Ok(DolangValue::Str(s.to_uppercase())),
        "lower" => Ok(DolangValue::Str(s.to_lowercase())),
        "trim" => Ok(DolangValue::Str(s.trim().to_string())),
        "contains" => {
            if args.is_empty() {
                return Err(Error::Interpreter("method 'contains' requires 1 argument".to_string()));
            }
            let arg = match &args[0] {
                DolangValue::Str(a) => a.as_str(),
                _ => return Err(Error::Interpreter("contains requires a String argument".to_string())),
            };
            Ok(DolangValue::Bool(s.contains(arg)))
        }
        "starts_with" => {
            if args.is_empty() {
                return Err(Error::Interpreter("method 'starts_with' requires 1 argument".to_string()));
            }
            let arg = match &args[0] {
                DolangValue::Str(a) => a.as_str(),
                _ => return Err(Error::Interpreter("starts_with requires a String argument".to_string())),
            };
            Ok(DolangValue::Bool(s.starts_with(arg)))
        }
        "ends_with" => {
            if args.is_empty() {
                return Err(Error::Interpreter("method 'ends_with' requires 1 argument".to_string()));
            }
            let arg = match &args[0] {
                DolangValue::Str(a) => a.as_str(),
                _ => return Err(Error::Interpreter("ends_with requires a String argument".to_string())),
            };
            Ok(DolangValue::Bool(s.ends_with(arg)))
        }
        "replace" => {
            if args.len() != 2 {
                return Err(Error::Interpreter("method 'replace' requires 2 arguments".to_string()));
            }
            let old = match &args[0] {
                DolangValue::Str(a) => a.as_str(),
                _ => return Err(Error::Interpreter("replace requires String arguments".to_string())),
            };
            let new = match &args[1] {
                DolangValue::Str(a) => a.as_str(),
                _ => return Err(Error::Interpreter("replace requires String arguments".to_string())),
            };
            Ok(DolangValue::Str(s.replace(old, new)))
        }
        "split" => {
            if args.is_empty() {
                return Err(Error::Interpreter("method 'split' requires 1 argument".to_string()));
            }
            let sep = match &args[0] {
                DolangValue::Str(a) => a.as_str(),
                _ => return Err(Error::Interpreter("split requires a String argument".to_string())),
            };
            let parts: Vec<DolangValue> = s.split(sep).map(|p| DolangValue::Str(p.to_string())).collect();
            Ok(DolangValue::List(parts))
        }
        "slice" => {
            if args.len() != 2 {
                return Err(Error::Interpreter("method 'slice' requires 2 arguments".to_string()));
            }
            let start = match &args[0] {
                DolangValue::Int(n) => *n as usize,
                _ => return Err(Error::Interpreter("slice requires Integer arguments".to_string())),
            };
            let end = match &args[1] {
                DolangValue::Int(n) => *n as usize,
                _ => return Err(Error::Interpreter("slice requires Integer arguments".to_string())),
            };
            let chars: Vec<char> = s.chars().collect();
            if start < chars.len() && end <= chars.len() && start < end {
                let slice: String = chars[start..end].iter().collect();
                Ok(DolangValue::Str(slice))
            } else {
                Err(Error::Interpreter("invalid slice indices".to_string()))
            }
        }
        "to_int" => {
            if let Ok(i) = s.parse::<i64>() {
                Ok(DolangValue::Int(i))
            } else if let Ok(f) = s.parse::<f64>() {
                Ok(DolangValue::Int(f as i64))
            } else {
                Err(Error::Interpreter(format!("cannot convert \"{}\" to Int", s)))
            }
        }
        "to_float" => {
            if let Ok(f) = s.parse::<f64>() {
                Ok(DolangValue::Float(f))
            } else {
                Err(Error::Interpreter(format!("cannot convert \"{}\" to Float", s)))
            }
        }
        "to_bool" => {
            if s == "true" {
                Ok(DolangValue::Bool(true))
            } else if s == "false" {
                Ok(DolangValue::Bool(false))
            } else {
                Err(Error::Interpreter(format!(
                    "cannot convert \"{}\" to Bool\nexpected \"true\" or \"false\"",
                    s
                )))
            }
        }
        "to_str" => Ok(receiver.clone()),
        "type" => Ok(DolangValue::Str("String".to_string())),
        _ => Err(Error::Interpreter(format!("string has no method '{}'", method))),
    }
}
