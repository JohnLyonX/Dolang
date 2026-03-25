// Map 内置方法实现
use crate::error::Error;
use crate::interpreter::DolangValue;

/// Map 方法分发 (不可变)
pub fn call(
    receiver: &DolangValue,
    method: &str,
    args: &[DolangValue],
) -> Result<DolangValue, Error> {
    let map = match receiver {
        DolangValue::Map(map) => map,
        _ => return Err(Error::Interpreter("expected Map".to_string())),
    };

    match method {
        "len" | "length" => Ok(DolangValue::Int(map.len() as i64)),
        "keys" => {
            let keys: Vec<DolangValue> = map.keys().map(|k| DolangValue::Str(k.clone())).collect();
            Ok(DolangValue::List(keys))
        }
        "values" => {
            let values: Vec<DolangValue> = map.values().cloned().collect();
            Ok(DolangValue::List(values))
        }
        "contains_key" => {
            if args.is_empty() {
                return Err(Error::Interpreter(
                    "method 'contains_key' requires 1 argument".to_string(),
                ));
            }
            let key = match &args[0] {
                DolangValue::Str(s) => s,
                _ => {
                    return Err(Error::Interpreter(
                        "contains_key requires a String argument".to_string(),
                    ));
                }
            };
            Ok(DolangValue::Bool(map.contains_key(key)))
        }
        "type" => Ok(DolangValue::Str("Map".to_string())),
        _ => Err(Error::Interpreter(format!(
            "map has no method '{}'",
            method
        ))),
    }
}

/// Map 方法分发 (可变)
pub fn call_mut(
    receiver: &mut DolangValue,
    method: &str,
    args: &[DolangValue],
) -> Result<DolangValue, Error> {
    let map = match receiver {
        DolangValue::Map(map) => map,
        _ => return Err(Error::Interpreter("expected Map".to_string())),
    };

    match method {
        "remove" => {
            if args.is_empty() {
                return Err(Error::Interpreter(
                    "method 'remove' requires 1 argument".to_string(),
                ));
            }
            let key = match &args[0] {
                DolangValue::Str(s) => s.clone(),
                _ => {
                    return Err(Error::Interpreter(
                        "remove requires a String argument".to_string(),
                    ));
                }
            };
            if map.swap_remove(&key).is_some() {
                Ok(receiver.clone())
            } else {
                Err(Error::Interpreter("key not found".to_string()))
            }
        }
        _ => call(receiver, method, args),
    }
}
