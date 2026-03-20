// List 内置方法实现
use crate::error::Error;
use crate::interpreter::DolangValue;

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
                return Err(Error::Interpreter(
                    "method 'contains' requires 1 argument".to_string(),
                ));
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
                    _ => {
                        return Err(Error::Interpreter(
                            "join requires a String argument".to_string(),
                        ));
                    }
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
        _ => Err(Error::Interpreter(format!(
            "list has no method '{}'",
            method
        ))),
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
                return Err(Error::Interpreter(
                    "method 'push' requires 1 argument".to_string(),
                ));
            }
            items.push(args[0].clone());
            Ok(receiver.clone())
        }
        "pop" => {
            if items.is_empty() {
                return Err(Error::Interpreter("cannot pop from empty list".to_string()));
            }
            items
                .pop()
                .ok_or_else(|| Error::Interpreter("pop failed".to_string()))
        }
        "reverse" => {
            items.reverse();
            Ok(receiver.clone())
        }
        _ => Err(Error::Interpreter(format!(
            "list has no mutable method '{}'",
            method
        ))),
    }
}
