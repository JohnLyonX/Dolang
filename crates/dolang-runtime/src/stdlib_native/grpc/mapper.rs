use crate::error::Error;
use crate::interpreter::DolangValue;
use indexmap::IndexMap;
use prost_reflect::{DynamicMessage, FieldDescriptor, Kind, MapKey, MessageDescriptor, Value};
use std::collections::HashMap;

pub fn encode_message(
    descriptor: MessageDescriptor,
    value: &DolangValue,
) -> Result<DynamicMessage, Error> {
    let map = match value {
        DolangValue::Map(map) | DolangValue::Json(map) => map,
        other => {
            return Err(Error::Interpreter(format!(
                "std.grpc: message '{}' expects Map or Json, got {}",
                descriptor.full_name(),
                other.type_name()
            )));
        }
    };

    let mut message = DynamicMessage::new(descriptor.clone());
    for (key, value) in map {
        let field = descriptor.get_field_by_name(key).ok_or_else(|| {
            Error::Interpreter(format!(
                "std.grpc: message '{}' has no field '{}'",
                descriptor.full_name(),
                key
            ))
        })?;
        reject_unsupported_field(&field)?;
        let Some(field_value) = encode_field_value(&field, value)? else {
            continue;
        };
        message.try_set_field(&field, field_value).map_err(|err| {
            Error::Interpreter(format!(
                "std.grpc: field '{}.{}' rejected value: {err:?}",
                descriptor.full_name(),
                field.name()
            ))
        })?;
    }

    Ok(message)
}

pub fn decode_message(message: &DynamicMessage) -> Result<DolangValue, Error> {
    let mut out = IndexMap::new();
    for (field, value) in message.fields() {
        reject_unsupported_field(&field)?;
        out.insert(field.name().to_string(), decode_field_value(value)?);
    }
    Ok(DolangValue::Map(out))
}

fn reject_unsupported_field(field: &FieldDescriptor) -> Result<(), Error> {
    if let Some(oneof) = field.containing_oneof() {
        if field
            .field_descriptor_proto()
            .proto3_optional
            .unwrap_or(false)
        {
            return Ok(());
        }
        return Err(Error::Interpreter(format!(
            "std.grpc: oneof field '{}.{}' in '{}' is not supported yet",
            field.parent_message().full_name(),
            field.name(),
            oneof.full_name()
        )));
    }

    if let Kind::Message(message) = field.kind() {
        reject_unsupported_message_descriptor(&message)?;
    }

    Ok(())
}

fn reject_unsupported_message_descriptor(descriptor: &MessageDescriptor) -> Result<(), Error> {
    let full_name = descriptor.full_name();
    if full_name == "google.protobuf.Any"
        || matches!(
            full_name,
            "google.protobuf.Timestamp"
                | "google.protobuf.Duration"
                | "google.protobuf.Struct"
                | "google.protobuf.Value"
                | "google.protobuf.ListValue"
                | "google.protobuf.FieldMask"
        )
    {
        return Err(Error::Interpreter(format!(
            "std.grpc: well-known type '{}' is not supported yet",
            full_name
        )));
    }

    Ok(())
}

fn encode_field_value(field: &FieldDescriptor, value: &DolangValue) -> Result<Option<Value>, Error> {
    if matches!(value, DolangValue::Null) && field.supports_presence() {
        return Ok(None);
    }

    if field.is_list() {
        let values = match value {
            DolangValue::List(values) => values,
            other => {
                return Err(Error::Interpreter(format!(
                    "std.grpc: field '{}.{}' expects List, got {}",
                    field.parent_message().full_name(),
                    field.name(),
                    other.type_name()
                )));
            }
        };

        let mut encoded = Vec::with_capacity(values.len());
        for value in values {
            encoded.push(encode_scalar_value(&field.kind(), value)?);
        }
        return Ok(Some(Value::List(encoded)));
    }

    if field.is_map() {
        let entries = match value {
            DolangValue::Map(entries) | DolangValue::Json(entries) => entries,
            other => {
                return Err(Error::Interpreter(format!(
                    "std.grpc: field '{}.{}' expects Map or Json, got {}",
                    field.parent_message().full_name(),
                    field.name(),
                    other.type_name()
                )));
            }
        };

        let map_entry = match field.kind() {
            Kind::Message(message) => message,
            _ => {
                return Err(Error::Interpreter(format!(
                    "std.grpc: field '{}.{}' is not a valid protobuf map",
                    field.parent_message().full_name(),
                    field.name()
                )));
            }
        };
        let key_kind = map_entry.map_entry_key_field().kind();
        let value_kind = map_entry.map_entry_value_field().kind();
        let mut encoded = HashMap::with_capacity(entries.len());
        for (key, value) in entries {
            encoded.insert(encode_map_key(&key_kind, key)?, encode_scalar_value(&value_kind, value)?);
        }
        return Ok(Some(Value::Map(encoded)));
    }

    Ok(Some(encode_scalar_value(&field.kind(), value)?))
}

fn encode_scalar_value(kind: &Kind, value: &DolangValue) -> Result<Value, Error> {
    match (kind, value) {
        (Kind::Bool, DolangValue::Bool(value)) => Ok(Value::Bool(*value)),
        (Kind::Int32 | Kind::Sint32 | Kind::Sfixed32, DolangValue::Int(value)) => {
            i32::try_from(*value)
                .map(Value::I32)
                .map_err(|_| Error::Interpreter(format!("std.grpc: value '{}' is out of range for i32", value)))
        }
        (Kind::Int64 | Kind::Sint64 | Kind::Sfixed64, DolangValue::Int(value)) => {
            Ok(Value::I64(*value))
        }
        (Kind::Uint32 | Kind::Fixed32, DolangValue::Int(value)) => u32::try_from(*value)
            .map(Value::U32)
            .map_err(|_| Error::Interpreter(format!("std.grpc: value '{}' is out of range for u32", value))),
        (Kind::Uint64 | Kind::Fixed64, DolangValue::Int(value)) => u64::try_from(*value)
            .map(Value::U64)
            .map_err(|_| Error::Interpreter(format!("std.grpc: value '{}' is out of range for u64", value))),
        (Kind::Float, DolangValue::Float(value)) => Ok(Value::F32(*value as f32)),
        (Kind::Double, DolangValue::Float(value)) => Ok(Value::F64(*value)),
        (Kind::String, DolangValue::Str(value)) => Ok(Value::String(value.clone())),
        (Kind::Bytes, DolangValue::Str(value)) => Ok(Value::Bytes(value.clone().into_bytes().into())),
        (Kind::Enum(enum_desc), DolangValue::Str(value)) => enum_desc
            .get_value_by_name(value)
            .map(|enum_value| Value::EnumNumber(enum_value.number()))
            .ok_or_else(|| {
                Error::Interpreter(format!(
                    "std.grpc: enum '{}' has no value '{}'",
                    enum_desc.full_name(),
                    value
                ))
            }),
        (Kind::Enum(enum_desc), DolangValue::Int(value)) => i32::try_from(*value)
            .map(Value::EnumNumber)
            .map_err(|_| {
                Error::Interpreter(format!(
                    "std.grpc: enum '{}' value '{}' is out of range",
                    enum_desc.full_name(),
                    value
                ))
            }),
        (Kind::Message(message_desc), DolangValue::Map(_) | DolangValue::Json(_)) => {
            encode_message(message_desc.clone(), value).map(Value::Message)
        }
        (Kind::Message(_), other) => Err(Error::Interpreter(format!(
            "std.grpc: nested message expects Map or Json, got {}",
            other.type_name()
        ))),
        (_, other) => Err(Error::Interpreter(format!(
            "std.grpc: unsupported field mapping from {}",
            other.type_name()
        ))),
    }
}

fn encode_map_key(kind: &Kind, key: &str) -> Result<MapKey, Error> {
    match kind {
        Kind::Bool => key.parse::<bool>().map(MapKey::Bool).map_err(|_| {
            Error::Interpreter(format!("std.grpc: map key '{}' is not a valid bool", key))
        }),
        Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 => {
            key.parse::<i32>().map(MapKey::I32).map_err(|_| {
                Error::Interpreter(format!("std.grpc: map key '{}' is not a valid i32", key))
            })
        }
        Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 => {
            key.parse::<i64>().map(MapKey::I64).map_err(|_| {
                Error::Interpreter(format!("std.grpc: map key '{}' is not a valid i64", key))
            })
        }
        Kind::Uint32 | Kind::Fixed32 => key.parse::<u32>().map(MapKey::U32).map_err(|_| {
            Error::Interpreter(format!("std.grpc: map key '{}' is not a valid u32", key))
        }),
        Kind::Uint64 | Kind::Fixed64 => key.parse::<u64>().map(MapKey::U64).map_err(|_| {
            Error::Interpreter(format!("std.grpc: map key '{}' is not a valid u64", key))
        }),
        Kind::String => Ok(MapKey::String(key.to_string())),
        _ => Err(Error::Interpreter(format!(
            "std.grpc: unsupported map key kind for '{}'",
            key
        ))),
    }
}

fn decode_field_value(value: &Value) -> Result<DolangValue, Error> {
    match value {
        Value::Bool(value) => Ok(DolangValue::Bool(*value)),
        Value::I32(value) => Ok(DolangValue::Int(i64::from(*value))),
        Value::I64(value) => Ok(DolangValue::Int(*value)),
        Value::U32(value) => Ok(DolangValue::Int(i64::from(*value))),
        Value::U64(value) => i64::try_from(*value).map(DolangValue::Int).map_err(|_| {
            Error::Interpreter(format!(
                "std.grpc: cannot decode u64 value '{}' into Dolang Int",
                value
            ))
        }),
        Value::F32(value) => Ok(DolangValue::Float(f64::from(*value))),
        Value::F64(value) => Ok(DolangValue::Float(*value)),
        Value::String(value) => Ok(DolangValue::Str(value.clone())),
        Value::Bytes(value) => Ok(DolangValue::Str(String::from_utf8_lossy(value).to_string())),
        Value::EnumNumber(value) => Ok(DolangValue::Int(i64::from(*value))),
        Value::Message(value) => decode_message(value),
        Value::List(values) => values
            .iter()
            .map(decode_field_value)
            .collect::<Result<Vec<_>, _>>()
            .map(DolangValue::List),
        Value::Map(values) => values
            .iter()
            .map(|(key, value)| Ok((decode_map_key(key), decode_field_value(value)?)))
            .collect::<Result<IndexMap<_, _>, Error>>()
            .map(DolangValue::Map),
    }
}

fn decode_map_key(key: &MapKey) -> String {
    match key {
        MapKey::Bool(value) => value.to_string(),
        MapKey::I32(value) => value.to_string(),
        MapKey::I64(value) => value.to_string(),
        MapKey::U32(value) => value.to_string(),
        MapKey::U64(value) => value.to_string(),
        MapKey::String(value) => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_message, encode_message};
    use crate::interpreter::DolangValue;
    use crate::stdlib_native::grpc_client::config::parse_client_config;
    use crate::stdlib_native::grpc_client::contract::GrpcContract;
    use indexmap::IndexMap;
    use prost_reflect::Value;
    use std::path::PathBuf;

    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
    }

    fn echo_method() -> prost_reflect::MethodDescriptor {
        method_from_descriptor(
            "tests/fixtures/grpc/descriptors/echo.pb",
            "dolang.test.echo.v1.EchoService",
            "Echo",
        )
    }

    fn types_method() -> prost_reflect::MethodDescriptor {
        method_from_descriptor(
            "tests/fixtures/grpc/descriptors/types.pb",
            "dolang.test.types.v1.TypesService",
            "RoundTrip",
        )
    }

    fn oneof_method() -> prost_reflect::MethodDescriptor {
        method_from_descriptor(
            "tests/fixtures/grpc/descriptors/oneof.pb",
            "dolang.test.oneof.v1.OneofService",
            "RoundTrip",
        )
    }

    fn wkt_method() -> prost_reflect::MethodDescriptor {
        method_from_descriptor(
            "tests/fixtures/grpc/descriptors/wkt.pb",
            "dolang.test.wkt.v1.WktService",
            "RoundTrip",
        )
    }

    fn method_from_descriptor(
        descriptor_path: &str,
        service: &str,
        method: &str,
    ) -> prost_reflect::MethodDescriptor {
        let config = parse_client_config(&[DolangValue::Map(IndexMap::from([
            (
                "target".to_string(),
                DolangValue::Str("dns:///echo-service:50051".to_string()),
            ),
            (
                "descriptor".to_string(),
                DolangValue::Str(descriptor_path.to_string()),
            ),
        ]))])
        .expect("config should parse");
        let contract = GrpcContract::load(&config, &fixture_root()).expect("descriptor should load");
        contract
            .method(service, method)
            .expect("method should exist")
    }

    #[test]
    fn encodes_map_into_dynamic_request_message() {
        let method = echo_method();
        let request = DolangValue::Map(IndexMap::from([(
            "message".to_string(),
            DolangValue::Str("hello grpc".to_string()),
        )]));

        let message = encode_message(method.input(), &request).expect("request should encode");

        assert_eq!(
            message
                .get_field_by_name("message")
                .expect("message field should exist")
                .as_ref(),
            &Value::String("hello grpc".to_string())
        );
    }

    #[test]
    fn rejects_unknown_request_field() {
        let method = echo_method();
        let request = DolangValue::Map(IndexMap::from([(
            "missing".to_string(),
            DolangValue::Str("hello grpc".to_string()),
        )]));

        let error = encode_message(method.input(), &request).expect_err("unknown field should fail");
        assert!(error.to_string().contains("has no field 'missing'"));
    }

    #[test]
    fn decodes_dynamic_response_message_into_map() {
        let method = echo_method();
        let response = encode_message(
            method.output(),
            &DolangValue::Map(IndexMap::from([(
                "message".to_string(),
                DolangValue::Str("pong".to_string()),
            )])),
        )
        .expect("response should encode");

        let value = decode_message(&response).expect("response should decode");
        assert_eq!(
            value,
            DolangValue::Map(IndexMap::from([(
                "message".to_string(),
                DolangValue::Str("pong".to_string()),
            )]))
        );
    }

    #[test]
    fn encodes_repeated_map_and_optional_fields() {
        let method = types_method();
        let request = DolangValue::Map(IndexMap::from([
            (
                "tags".to_string(),
                DolangValue::List(vec![
                    DolangValue::Str("alpha".to_string()),
                    DolangValue::Str("beta".to_string()),
                ]),
            ),
            (
                "counters".to_string(),
                DolangValue::Map(IndexMap::from([
                    ("a".to_string(), DolangValue::Int(1)),
                    ("b".to_string(), DolangValue::Int(2)),
                ])),
            ),
            (
                "nested".to_string(),
                DolangValue::Map(IndexMap::from([(
                    "label".to_string(),
                    DolangValue::Str("root".to_string()),
                )])),
            ),
            (
                "status".to_string(),
                DolangValue::Str("STATUS_READY".to_string()),
            ),
            (
                "alias".to_string(),
                DolangValue::Str("visible".to_string()),
            ),
            (
                "items".to_string(),
                DolangValue::List(vec![
                    DolangValue::Map(IndexMap::from([(
                        "label".to_string(),
                        DolangValue::Str("one".to_string()),
                    )])),
                    DolangValue::Map(IndexMap::from([(
                        "label".to_string(),
                        DolangValue::Str("two".to_string()),
                    )])),
                ]),
            ),
            (
                "lookup".to_string(),
                DolangValue::Map(IndexMap::from([
                    (
                        "left".to_string(),
                        DolangValue::Map(IndexMap::from([(
                            "label".to_string(),
                            DolangValue::Str("L".to_string()),
                        )])),
                    ),
                    (
                        "right".to_string(),
                        DolangValue::Map(IndexMap::from([(
                            "label".to_string(),
                            DolangValue::Str("R".to_string()),
                        )])),
                    ),
                ])),
            ),
        ]));

        let message = encode_message(method.input(), &request).expect("request should encode");

        assert_eq!(
            message.get_field_by_name("tags").expect("tags field").as_ref(),
            &Value::List(vec![
                Value::String("alpha".to_string()),
                Value::String("beta".to_string()),
            ])
        );
        assert_eq!(
            message
                .get_field_by_name("alias")
                .expect("alias field should exist")
                .as_ref(),
            &Value::String("visible".to_string())
        );
        assert_eq!(
            message
                .get_field_by_name("status")
                .expect("status field")
                .as_ref(),
            &Value::EnumNumber(1)
        );
    }

    #[test]
    fn encodes_optional_null_as_absent_field() {
        let method = types_method();
        let request = DolangValue::Map(IndexMap::from([(
            "alias".to_string(),
            DolangValue::Null,
        )]));

        let message = encode_message(method.input(), &request).expect("optional null should encode");

        let alias = method
            .input()
            .get_field_by_name("alias")
            .expect("alias field descriptor should exist");
        assert!(!message.has_field(&alias));
    }

    #[test]
    fn rejects_non_list_for_repeated_field() {
        let method = types_method();
        let request = DolangValue::Map(IndexMap::from([(
            "tags".to_string(),
            DolangValue::Str("alpha".to_string()),
        )]));

        let error = encode_message(method.input(), &request).expect_err("non-list should fail");
        assert!(error.to_string().contains("expects List"));
    }

    #[test]
    fn decodes_repeated_map_and_nested_fields() {
        let method = types_method();
        let response = encode_message(
            method.output(),
            &DolangValue::Map(IndexMap::from([
                (
                    "tags".to_string(),
                    DolangValue::List(vec![
                        DolangValue::Str("alpha".to_string()),
                        DolangValue::Str("beta".to_string()),
                    ]),
                ),
                (
                    "counters".to_string(),
                    DolangValue::Map(IndexMap::from([
                        ("a".to_string(), DolangValue::Int(1)),
                        ("b".to_string(), DolangValue::Int(2)),
                    ])),
                ),
                (
                    "nested".to_string(),
                    DolangValue::Map(IndexMap::from([(
                        "label".to_string(),
                        DolangValue::Str("root".to_string()),
                    )])),
                ),
                (
                    "items".to_string(),
                    DolangValue::List(vec![DolangValue::Map(IndexMap::from([(
                        "label".to_string(),
                        DolangValue::Str("one".to_string()),
                    )]))]),
                ),
                (
                    "lookup".to_string(),
                    DolangValue::Map(IndexMap::from([(
                        "left".to_string(),
                        DolangValue::Map(IndexMap::from([(
                            "label".to_string(),
                            DolangValue::Str("L".to_string()),
                        )])),
                    )])),
                ),
                (
                    "alias".to_string(),
                    DolangValue::Str("visible".to_string()),
                ),
            ])),
        )
        .expect("response should encode");

        let value = decode_message(&response).expect("response should decode");

        match value {
            DolangValue::Map(map) => {
                assert_eq!(
                    map.get("tags"),
                    Some(&DolangValue::List(vec![
                        DolangValue::Str("alpha".to_string()),
                        DolangValue::Str("beta".to_string()),
                    ]))
                );
                assert_eq!(
                    map.get("counters"),
                    Some(&DolangValue::Map(IndexMap::from([
                        ("a".to_string(), DolangValue::Int(1)),
                        ("b".to_string(), DolangValue::Int(2)),
                    ])))
                );
                assert_eq!(
                    map.get("alias"),
                    Some(&DolangValue::Str("visible".to_string()))
                );
            }
            other => panic!("expected map, got {}", other.type_name()),
        }
    }

    #[test]
    fn rejects_oneof_request_fields_with_explicit_error() {
        let method = oneof_method();
        let request = DolangValue::Map(IndexMap::from([(
            "text".to_string(),
            DolangValue::Str("hello".to_string()),
        )]));

        let error = encode_message(method.input(), &request).expect_err("oneof should fail");
        assert!(error.to_string().contains("oneof"));
        assert!(error.to_string().contains("not supported"));
    }

    #[test]
    fn rejects_decoding_messages_with_oneof_fields() {
        let method = oneof_method();
        let response = encode_message(
            method.output(),
            &DolangValue::Map(IndexMap::from([(
                "text".to_string(),
                DolangValue::Str("hello".to_string()),
            )])),
        );

        let error = response.expect_err("oneof encoding should fail before decode");
        assert!(error.to_string().contains("oneof"));
        assert!(error.to_string().contains("not supported"));
    }

    #[test]
    fn rejects_any_with_explicit_error() {
        let method = wkt_method();
        let request = DolangValue::Map(IndexMap::from([(
            "payload".to_string(),
            DolangValue::Map(IndexMap::new()),
        )]));

        let error = encode_message(method.input(), &request).expect_err("Any should fail");
        assert!(error.to_string().contains("google.protobuf.Any"));
        assert!(error.to_string().contains("not supported"));
    }

    #[test]
    fn rejects_complex_well_known_types_with_explicit_error() {
        let method = wkt_method();
        let request = DolangValue::Map(IndexMap::from([(
            "created_at".to_string(),
            DolangValue::Map(IndexMap::from([(
                "seconds".to_string(),
                DolangValue::Int(1),
            )])),
        )]));

        let error =
            encode_message(method.input(), &request).expect_err("Timestamp should fail");
        assert!(error.to_string().contains("google.protobuf.Timestamp"));
        assert!(error.to_string().contains("not supported"));
    }
}
