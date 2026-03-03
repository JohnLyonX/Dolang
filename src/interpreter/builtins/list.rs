// List 内置方法实现
use crate::error::Error;
use crate::interpreter::DolangValue;
use crate::interpreter::env::{deserialize_list, serialize_list};

// ===== 字符串版本（过渡期）=====

/// List 方法分发（字符串版本 - 过渡期）
/// obj_val: list 的序列化字符串形式 (__LST__:...)
/// method: 方法名
/// args: 方法参数
pub fn call_str(
    obj_val: &str,
    method: &str,
    args: &[String],
) -> Result<String, Error> {
    match method {
        "len" | "length" => {
            let len = deserialize_list(obj_val).map(|l| l.len()).unwrap_or(0);
            Ok(len.to_string())
        }
        "push" => {
            if args.is_empty() {
                return Err(Error::Interpreter("method 'push' requires 1 argument".to_string()));
            }
            let mut list = deserialize_list(obj_val).unwrap_or_default();
            list.push(args[0].clone());
            Ok(serialize_list(&list))
        }
        "pop" => {
            let mut list = deserialize_list(obj_val).unwrap_or_default();
            if list.is_empty() {
                return Err(Error::Interpreter("cannot pop from empty list".to_string()));
            }
            let popped = list.pop().unwrap();
            Ok(popped)
        }
        "contains" => {
            if args.is_empty() {
                return Err(Error::Interpreter("method 'contains' requires 1 argument".to_string()));
            }
            let list = deserialize_list(obj_val).unwrap_or_default();
            Ok(list.contains(&args[0]).to_string())
        }
        "reverse" => {
            let mut list = deserialize_list(obj_val).unwrap_or_default();
            list.reverse();
            Ok(serialize_list(&list))
        }
        "join" => {
            let sep = if args.is_empty() {
                String::new()
            } else {
                args[0].clone()
            };
            let list = deserialize_list(obj_val).unwrap_or_default();
            Ok(list.join(&sep))
        }
        "type" => {
            Ok("List".to_string())
        }
        _ => {
            Err(Error::Interpreter(format!("list has no method '{}'", method)))
        }
    }
}

/// List 方法分发（可变，字符串版本）
pub fn call_mut_str(
    obj_val: &str,
    method: &str,
    args: &[String],
) -> Result<String, Error> {
    call_str(obj_val, method, args)
}

// ===== DolangValue 版本 =====

/// List 方法分发（不可变）
pub fn call(
    receiver: &DolangValue,
    method: &str,
    args: &[DolangValue],
) -> Result<DolangValue, Error> {
    let items = match receiver {
        DolangValue::List(items) => items,
        _ => return Err(Error::Interpreter("expected List".to_string())),
    };

    match method {
        "len" | "length" => Ok(DolangValue::Int(items.len() as i64)),
        "contains" => {
            if args.is_empty() {
                return Err(Error::Interpreter("method 'contains' requires 1 argument".to_string()));
            }
            let contains = items.iter().any(|item| item == &args[0]);
            Ok(DolangValue::Bool(contains))
        }
        "join" => {
            let sep = if args.is_empty() {
                String::new()
            } else {
                match &args[0] {
                    DolangValue::Str(s) => s.clone(),
                    _ => return Err(Error::Interpreter("join requires a String argument".to_string())),
                }
            };
            let result: String = items
                .iter()
                .map(|item| item.to_string())
                .collect::<Vec<_>>()
                .join(&sep);
            Ok(DolangValue::Str(result))
        }
        "type" => Ok(DolangValue::Str("List".to_string())),
        _ => {
            Err(Error::Interpreter(format!("list has no method '{}'", method)))
        }
    }
}

/// List 方法分发（可变）
pub fn call_mut(
    receiver: &mut DolangValue,
    method: &str,
    args: &[DolangValue],
) -> Result<DolangValue, Error> {
    let items = match receiver {
        DolangValue::List(items) => items,
        _ => return Err(Error::Interpreter("expected List".to_string())),
    };

    match method {
        "push" => {
            if args.is_empty() {
                return Err(Error::Interpreter("method 'push' requires 1 argument".to_string()));
            }
            items.push(args[0].clone());
            Ok(receiver.clone())
        }
        "pop" => {
            if items.is_empty() {
                return Err(Error::Interpreter("cannot pop from empty list".to_string()));
            }
            items.pop().ok_or_else(|| Error::Interpreter("pop failed".to_string()))
        }
        "reverse" => {
            items.reverse();
            Ok(receiver.clone())
        }
        _ => {
            Err(Error::Interpreter(format!("list has no mutable method '{}'", method)))
        }
    }
}
