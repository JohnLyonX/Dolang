use crate::error::Error;
use crate::interpreter::DolangValue;
use indexmap::IndexMap;

use std::collections::HashMap;

use super::types::{GrpcCallConfig, GrpcClientConfig, GrpcContractKind};

pub fn parse_client_config(args: &[DolangValue]) -> Result<GrpcClientConfig, Error> {
    if args.len() != 1 {
        return Err(Error::Interpreter(format!(
            "std.grpc.client() requires 1 argument (config), got {}",
            args.len()
        )));
    }

    let config = match &args[0] {
        DolangValue::Map(map) | DolangValue::Json(map) => map,
        other => {
            return Err(Error::Interpreter(format!(
                "std.grpc.client() expects Map at argument 0, got {}",
                other.type_name()
            )));
        }
    };

    let target = required_string_field("std.grpc.client", config, "target")?;
    let descriptor = optional_string_field("std.grpc.client", config, "descriptor")?;
    let proto = optional_string_field("std.grpc.client", config, "proto")?;

    match (descriptor, proto) {
        (Some(descriptor), None) => Ok(GrpcClientConfig {
            target,
            contract_path: descriptor,
            contract_kind: GrpcContractKind::Descriptor,
            timeout_ms: optional_u64_field("std.grpc.client", config, "timeout_ms")?,
            metadata: optional_string_map_field("std.grpc.client", config, "metadata")?,
        }),
        (None, Some(proto)) => Ok(GrpcClientConfig {
            target,
            contract_path: proto,
            contract_kind: GrpcContractKind::Proto,
            timeout_ms: optional_u64_field("std.grpc.client", config, "timeout_ms")?,
            metadata: optional_string_map_field("std.grpc.client", config, "metadata")?,
        }),
        (Some(_), Some(_)) => Err(Error::Interpreter(
            "std.grpc.client() accepts either 'descriptor' or 'proto', not both".to_string(),
        )),
        (None, None) => Err(Error::Interpreter(
            "std.grpc.client() requires either 'descriptor' or 'proto'".to_string(),
        )),
    }
}

pub fn parse_call_config(args: &[DolangValue]) -> Result<GrpcCallConfig, Error> {
    if args.len() != 1 {
        return Err(Error::Interpreter(format!(
            "GrpcClient.call() requires 1 argument (config), got {}",
            args.len()
        )));
    }

    let config = match &args[0] {
        DolangValue::Map(map) | DolangValue::Json(map) => map,
        other => {
            return Err(Error::Interpreter(format!(
                "GrpcClient.call() expects Map at argument 0, got {}",
                other.type_name()
            )));
        }
    };

    Ok(GrpcCallConfig {
        service: required_string_field("GrpcClient.call", config, "service")?,
        method: required_string_field("GrpcClient.call", config, "method")?,
        body: required_value_field("GrpcClient.call", config, "body")?,
        timeout_ms: optional_u64_field("GrpcClient.call", config, "timeout_ms")?,
        metadata: optional_string_map_field("GrpcClient.call", config, "metadata")?,
    })
}

fn required_string_field(
    id: &str,
    map: &IndexMap<String, DolangValue>,
    key: &str,
) -> Result<String, Error> {
    match map.get(key) {
        Some(DolangValue::Str(value)) => Ok(value.clone()),
        Some(other) => Err(Error::Interpreter(format!(
            "{id} config field '{key}' must be String, got {}",
            other.type_name()
        ))),
        None => Err(Error::Interpreter(format!(
            "{id} config requires '{key}'"
        ))),
    }
}

fn optional_string_field(
    id: &str,
    map: &IndexMap<String, DolangValue>,
    key: &str,
) -> Result<Option<String>, Error> {
    match map.get(key) {
        Some(DolangValue::Str(value)) => Ok(Some(value.clone())),
        Some(other) => Err(Error::Interpreter(format!(
            "{id} config field '{key}' must be String, got {}",
            other.type_name()
        ))),
        None => Ok(None),
    }
}

fn required_value_field(
    id: &str,
    map: &IndexMap<String, DolangValue>,
    key: &str,
) -> Result<DolangValue, Error> {
    map.get(key)
        .cloned()
        .ok_or_else(|| Error::Interpreter(format!("{id} config requires '{key}'")))
}

fn optional_u64_field(
    id: &str,
    map: &IndexMap<String, DolangValue>,
    key: &str,
) -> Result<Option<u64>, Error> {
    match map.get(key) {
        Some(DolangValue::Int(value)) => u64::try_from(*value).map(Some).map_err(|_| {
            Error::Interpreter(format!(
                "{id} config field '{key}' must be a non-negative Int"
            ))
        }),
        Some(other) => Err(Error::Interpreter(format!(
            "{id} config field '{key}' must be Int, got {}",
            other.type_name()
        ))),
        None => Ok(None),
    }
}

fn optional_string_map_field(
    id: &str,
    map: &IndexMap<String, DolangValue>,
    key: &str,
) -> Result<HashMap<String, String>, Error> {
    match map.get(key) {
        Some(DolangValue::Map(entries)) | Some(DolangValue::Json(entries)) => entries
            .iter()
            .map(|(name, value)| match value {
                DolangValue::Str(value) => Ok((name.clone(), value.clone())),
                other => Err(Error::Interpreter(format!(
                    "{id} config field '{key}.{name}' must be String, got {}",
                    other.type_name()
                ))),
            })
            .collect(),
        Some(other) => Err(Error::Interpreter(format!(
            "{id} config field '{key}' must be Map, got {}",
            other.type_name()
        ))),
        None => Ok(HashMap::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_call_config, parse_client_config};
    use crate::interpreter::DolangValue;
    use indexmap::IndexMap;

    #[test]
    fn parse_client_config_rejects_both_descriptor_and_proto() {
        let mut config = IndexMap::new();
        config.insert(
            "target".to_string(),
            DolangValue::Str("dns:///users:50051".to_string()),
        );
        config.insert(
            "descriptor".to_string(),
            DolangValue::Str("descriptors/users.pb".to_string()),
        );
        config.insert(
            "proto".to_string(),
            DolangValue::Str("proto/users.proto".to_string()),
        );

        let error = parse_client_config(&[DolangValue::Map(config)])
            .expect_err("descriptor and proto together should fail");

        assert!(error.to_string().contains("descriptor"));
        assert!(error.to_string().contains("proto"));
    }

    #[test]
    fn parse_call_config_requires_service_method_and_body() {
        let mut config = IndexMap::new();
        config.insert(
            "service".to_string(),
            DolangValue::Str("dolang.test.echo.v1.EchoService".to_string()),
        );
        let error = parse_call_config(&[DolangValue::Map(config)])
            .expect_err("missing method/body should fail");
        assert!(error.to_string().contains("method"));
    }
}
