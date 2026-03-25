use std::sync::Arc;

use indexmap::IndexMap;

use crate::error::Error;
use crate::interpreter::DolangValue;
use crate::interpreter::value::value_to_json;
use crate::runtime::intrinsics::intrinsic_string_arg;
use crate::runtime::{NativeFnMap, RuntimeContext};

pub fn register(context: &mut RuntimeContext) {
    let mut exports = NativeFnMap::new();
    exports.insert(
        "parse".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let s = intrinsic_string_arg("json.parse", args, 0)?;
            let value: serde_json::Value = serde_json::from_str(s)
                .map_err(|err| Error::Interpreter(format!("json.parse: {err}")))?;
            Ok(json_value_to_dolang(value))
        }),
    );
    exports.insert(
        "stringify".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let Some(value) = args.first() else {
                return Err(Error::Interpreter(
                    "json.stringify: missing first arg".into(),
                ));
            };
            Ok(DolangValue::Str(value_to_json(value).to_string()))
        }),
    );
    exports.insert(
        "get".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let object = args
                .first()
                .ok_or_else(|| Error::Interpreter("json.get: missing first arg".into()))?;
            let key = intrinsic_string_arg("json.get", args, 1)?;
            match object {
                DolangValue::Map(map) | DolangValue::Json(map) => {
                    Ok(map.get(key).cloned().unwrap_or(DolangValue::Null))
                }
                other => Err(Error::Interpreter(format!(
                    "json.get: first arg must be Map or Json, got {}",
                    other.type_name()
                ))),
            }
        }),
    );
    exports.insert(
        "has".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let object = args
                .first()
                .ok_or_else(|| Error::Interpreter("json.has: missing first arg".into()))?;
            let key = intrinsic_string_arg("json.has", args, 1)?;
            match object {
                DolangValue::Map(map) | DolangValue::Json(map) => {
                    Ok(DolangValue::Bool(map.contains_key(key)))
                }
                other => Err(Error::Interpreter(format!(
                    "json.has: first arg must be Map or Json, got {}",
                    other.type_name()
                ))),
            }
        }),
    );
    exports.insert(
        "keys".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let object = args
                .first()
                .ok_or_else(|| Error::Interpreter("json.keys: missing first arg".into()))?;
            match object {
                DolangValue::Map(map) | DolangValue::Json(map) => Ok(DolangValue::List(
                    map.keys().map(|k| DolangValue::Str(k.clone())).collect(),
                )),
                other => Err(Error::Interpreter(format!(
                    "json.keys: first arg must be Map or Json, got {}",
                    other.type_name()
                ))),
            }
        }),
    );
    exports.insert(
        "values".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let object = args
                .first()
                .ok_or_else(|| Error::Interpreter("json.values: missing first arg".into()))?;
            match object {
                DolangValue::Map(map) | DolangValue::Json(map) => {
                    Ok(DolangValue::List(map.values().cloned().collect()))
                }
                other => Err(Error::Interpreter(format!(
                    "json.values: first arg must be Map or Json, got {}",
                    other.type_name()
                ))),
            }
        }),
    );
    exports.insert(
        "set".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let object = args
                .first()
                .ok_or_else(|| Error::Interpreter("json.set: missing first arg".into()))?;
            let key = intrinsic_string_arg("json.set", args, 1)?.to_string();
            let value = args
                .get(2)
                .ok_or_else(|| Error::Interpreter("json.set: missing third arg".into()))?
                .clone();
            match object {
                DolangValue::Map(map) | DolangValue::Json(map) => {
                    let mut new_map = map.clone();
                    new_map.insert(key, value);
                    Ok(DolangValue::Map(new_map))
                }
                other => Err(Error::Interpreter(format!(
                    "json.set: first arg must be Map or Json, got {}",
                    other.type_name()
                ))),
            }
        }),
    );
    exports.insert(
        "delete".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let object = args
                .first()
                .ok_or_else(|| Error::Interpreter("json.delete: missing first arg".into()))?;
            let key = intrinsic_string_arg("json.delete", args, 1)?;
            match object {
                DolangValue::Map(map) | DolangValue::Json(map) => {
                    let mut new_map = map.clone();
                    new_map.shift_remove(key);
                    Ok(DolangValue::Map(new_map))
                }
                other => Err(Error::Interpreter(format!(
                    "json.delete: first arg must be Map or Json, got {}",
                    other.type_name()
                ))),
            }
        }),
    );
    exports.insert(
        "pretty".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let value = args
                .first()
                .ok_or_else(|| Error::Interpreter("json.pretty: missing first arg".into()))?;
            let json = value_to_json(value);
            serde_json::to_string_pretty(&json)
                .map(DolangValue::Str)
                .map_err(|err| Error::Interpreter(format!("json.pretty: {err}")))
        }),
    );
    exports.insert(
        "merge".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| {
            let a = args
                .first()
                .ok_or_else(|| Error::Interpreter("json.merge: missing first arg".into()))?;
            let b = args
                .get(1)
                .ok_or_else(|| Error::Interpreter("json.merge: missing second arg".into()))?;
            match (a, b) {
                (
                    DolangValue::Map(ma) | DolangValue::Json(ma),
                    DolangValue::Map(mb) | DolangValue::Json(mb),
                ) => {
                    let mut merged = ma.clone();
                    merged.extend(mb.iter().map(|(k, v)| (k.clone(), v.clone())));
                    Ok(DolangValue::Map(merged))
                }
                _ => Err(Error::Interpreter(format!(
                    "json.merge: both args must be Map or Json, got {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            }
        }),
    );
    context.register_native_module("std.json", exports);
}

fn json_value_to_dolang(value: serde_json::Value) -> DolangValue {
    match value {
        serde_json::Value::Null => DolangValue::Null,
        serde_json::Value::Bool(b) => DolangValue::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                DolangValue::Int(i)
            } else if let Some(f) = n.as_f64() {
                DolangValue::Float(f)
            } else {
                DolangValue::Null
            }
        }
        serde_json::Value::String(s) => DolangValue::Str(s),
        serde_json::Value::Array(values) => {
            DolangValue::List(values.into_iter().map(json_value_to_dolang).collect())
        }
        serde_json::Value::Object(entries) => DolangValue::Map(
            entries
                .into_iter()
                .map(|(key, value)| (key, json_value_to_dolang(value)))
                .collect::<IndexMap<_, _>>(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::json_value_to_dolang;
    use crate::interpreter::DolangValue;

    #[test]
    fn converts_json_objects_to_maps() {
        let value = serde_json::json!({
            "name": "dolang",
            "nested": { "ok": true },
            "items": [1, 2]
        });
        let converted = json_value_to_dolang(value);
        match converted {
            DolangValue::Map(map) => {
                assert_eq!(
                    map.get("name"),
                    Some(&DolangValue::Str("dolang".to_string()))
                );
                assert!(matches!(map.get("nested"), Some(DolangValue::Map(_))));
                assert!(matches!(map.get("items"), Some(DolangValue::List(_))));
            }
            other => panic!("expected Map, got {}", other.type_name()),
        }
    }
}
