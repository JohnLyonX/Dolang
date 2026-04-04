use indexmap::IndexMap;

use crate::runtime::RuntimeContext;

use super::value::DolangValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeValidationError {
    BareList,
    UnknownType { expected_type: String },
    Mismatch { expected_type: String, actual_type: String },
    MissingRequiredField { type_name: String, field_name: String },
    UnknownField { type_name: String, field_name: String },
}

pub fn validate_value_against_type(
    expected_type: &str,
    value: &DolangValue,
    context: &RuntimeContext,
) -> Result<(), TypeValidationError> {
    let expected_type = expected_type.trim();

    if let Some(item_type) = parse_list_item_type(expected_type) {
        let DolangValue::List(items) = value else {
            return Err(TypeValidationError::Mismatch {
                expected_type: expected_type.to_string(),
                actual_type: value.type_name().into_owned(),
            });
        };

        for item in items {
            validate_value_against_type(item_type, item, context)?;
        }

        return Ok(());
    }

    let normalized = expected_type.to_ascii_lowercase();
    match normalized.as_str() {
        "int" | "integer" => expect_int(expected_type, value),
        "float" => expect_float(expected_type, value),
        "string" | "str" => expect_string(expected_type, value),
        "bool" | "boolean" => expect_bool(expected_type, value),
        "list" => Err(TypeValidationError::BareList),
        "map" => expect_map(expected_type, value),
        "response" => expect_response(expected_type, value),
        "json" => expect_json_like(expected_type, value, context),
        _ => expect_user_type(expected_type, value, context),
    }
}

pub fn validate_typed_instance_fields(
    type_name: &str,
    fields: &IndexMap<String, DolangValue>,
    context: &RuntimeContext,
) -> Result<(), TypeValidationError> {
    let shape = context
        .get_type(type_name)
        .ok_or_else(|| TypeValidationError::UnknownType {
            expected_type: type_name.to_string(),
        })?;

    for (stored_key, value) in fields {
        let field_name = stored_key
            .strip_prefix('_')
            .unwrap_or(stored_key.as_str())
            .to_string();
        let Some(field) = shape.fields.iter().find(|field| field.name == field_name) else {
            return Err(TypeValidationError::UnknownField {
                type_name: type_name.to_string(),
                field_name,
            });
        };

        validate_value_against_type(&field.type_name, value, context)?;
    }

    for field in &shape.fields {
        let stored_name = if field.hidden {
            format!("_{}", field.name)
        } else {
            field.name.clone()
        };
        if !field.optional && !fields.contains_key(&stored_name) {
            return Err(TypeValidationError::MissingRequiredField {
                type_name: type_name.to_string(),
                field_name: field.name.clone(),
            });
        }
    }

    Ok(())
}

pub fn parse_list_item_type(expected_type: &str) -> Option<&str> {
    let expected_type = expected_type.trim();
    let rest = expected_type.strip_prefix("List<")?;
    rest.strip_suffix('>').map(str::trim)
}

fn expect_int(expected_type: &str, value: &DolangValue) -> Result<(), TypeValidationError> {
    if matches!(value, DolangValue::Int(_)) {
        Ok(())
    } else {
        mismatch(expected_type, value)
    }
}

fn expect_float(expected_type: &str, value: &DolangValue) -> Result<(), TypeValidationError> {
    if matches!(value, DolangValue::Float(_) | DolangValue::Int(_)) {
        Ok(())
    } else {
        mismatch(expected_type, value)
    }
}

fn expect_string(expected_type: &str, value: &DolangValue) -> Result<(), TypeValidationError> {
    if matches!(value, DolangValue::Str(_)) {
        Ok(())
    } else {
        mismatch(expected_type, value)
    }
}

fn expect_bool(expected_type: &str, value: &DolangValue) -> Result<(), TypeValidationError> {
    if matches!(value, DolangValue::Bool(_)) {
        Ok(())
    } else {
        mismatch(expected_type, value)
    }
}

fn expect_map(expected_type: &str, value: &DolangValue) -> Result<(), TypeValidationError> {
    if matches!(value, DolangValue::Map(_)) {
        Ok(())
    } else {
        mismatch(expected_type, value)
    }
}

fn expect_response(expected_type: &str, value: &DolangValue) -> Result<(), TypeValidationError> {
    if matches!(value, DolangValue::Response { .. }) {
        Ok(())
    } else {
        mismatch(expected_type, value)
    }
}

fn expect_json_like(
    expected_type: &str,
    value: &DolangValue,
    context: &RuntimeContext,
) -> Result<(), TypeValidationError> {
    if let DolangValue::Response {
        body: Some(body), ..
    } = value
    {
        return expect_json_like(expected_type, body.as_ref(), context);
    }

    if matches!(
        value,
        DolangValue::Json(_) | DolangValue::Map(_) | DolangValue::TypedInstance { .. }
    ) {
        Ok(())
    } else {
        let _ = context;
        mismatch(expected_type, value)
    }
}

fn expect_user_type(
    expected_type: &str,
    value: &DolangValue,
    context: &RuntimeContext,
) -> Result<(), TypeValidationError> {
    if context.get_type(expected_type).is_none() {
        return Err(TypeValidationError::UnknownType {
            expected_type: expected_type.to_string(),
        });
    }

    match value {
        DolangValue::TypedInstance { type_name, .. } if type_name == expected_type => Ok(()),
        _ => mismatch(expected_type, value),
    }
}

fn mismatch(expected_type: &str, value: &DolangValue) -> Result<(), TypeValidationError> {
    Err(TypeValidationError::Mismatch {
        expected_type: expected_type.to_string(),
        actual_type: value.type_name().into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::runtime::{
        RuntimeMode,
        context::{TypeField, TypeShape},
    };

    fn test_context() -> RuntimeContext {
        RuntimeContext::new(RuntimeMode::Test, PathBuf::from("."))
    }

    #[test]
    fn validate_value_against_type_rejects_map_for_user_type() {
        let mut context = test_context();
        context.register_type(
            "User",
            TypeShape {
                name: "User".to_string(),
                fields: vec![],
            },
        );

        let value = DolangValue::Map(IndexMap::new());
        let error = validate_value_against_type("User", &value, &context)
            .expect_err("map should not satisfy User");

        assert_eq!(
            error,
            TypeValidationError::Mismatch {
                expected_type: "User".to_string(),
                actual_type: "Map".to_string(),
            }
        );
    }

    #[test]
    fn validate_typed_instance_fields_requires_non_optional_fields() {
        let mut context = test_context();
        context.register_type(
            "User",
            TypeShape {
                name: "User".to_string(),
                fields: vec![TypeField {
                    name: "id".to_string(),
                    type_name: "Int".to_string(),
                    optional: false,
                    hidden: false,
                }],
            },
        );

        let error = validate_typed_instance_fields("User", &IndexMap::new(), &context)
            .expect_err("missing required field should fail");

        assert_eq!(
            error,
            TypeValidationError::MissingRequiredField {
                type_name: "User".to_string(),
                field_name: "id".to_string(),
            }
        );
    }

    #[test]
    fn validate_value_against_type_accepts_list_of_user_instances() {
        let mut context = test_context();
        context.register_type(
            "User",
            TypeShape {
                name: "User".to_string(),
                fields: vec![],
            },
        );

        let value = DolangValue::List(vec![DolangValue::TypedInstance {
            type_name: "User".to_string(),
            fields: IndexMap::new(),
        }]);

        assert!(validate_value_against_type("List<User>", &value, &context).is_ok());
    }
}
