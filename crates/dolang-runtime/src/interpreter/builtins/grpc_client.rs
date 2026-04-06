use crate::error::Error;
use crate::interpreter::DolangValue;
use crate::runtime::RuntimeContext;
use crate::stdlib_native::grpc_client::config::parse_call_config;
use crate::stdlib_native::grpc_client::contract::GrpcContract;
use crate::stdlib_native::grpc_client::mapper::{decode_message, encode_message};
use crate::stdlib_native::grpc_client::transport::{execute_unary, metadata_map_to_hash_map};
use crate::stdlib_native::grpc_client::types::GrpcClientConfig;
use indexmap::IndexMap;

pub fn call(
    receiver: &DolangValue,
    method: &str,
    args: &[DolangValue],
    context: &RuntimeContext,
) -> Result<DolangValue, Error> {
    let client = match receiver {
        DolangValue::GrpcClient {
            target,
            contract_path,
            contract_kind,
            timeout_ms,
            metadata,
        } => GrpcClientConfig {
            target: target.clone(),
            contract_path: contract_path.clone(),
            contract_kind: contract_kind.clone(),
            timeout_ms: *timeout_ms,
            metadata: metadata
                .iter()
                .filter_map(|(key, value)| match value {
                    DolangValue::Str(value) => Some((key.clone(), value.clone())),
                    _ => None,
                })
                .collect(),
        },
        _ => return Err(Error::Interpreter("expected GrpcClient".to_string())),
    };

    match method {
        "call" => call_unary(client, args, context),
        _ => Err(Error::Interpreter(format!(
            "GrpcClient has no method '{}'",
            method
        ))),
    }
}

fn call_unary(
    client: GrpcClientConfig,
    args: &[DolangValue],
    context: &RuntimeContext,
) -> Result<DolangValue, Error> {
    let call = parse_call_config(args)?;
    let contract = GrpcContract::load(&client, context.project_root())?;
    let method = contract.method(&call.service, &call.method)?;
    let request = encode_message(method.input(), &call.body)?;
    let response = execute_unary(&client, &call, &method, request)?;

    match response {
        Ok(response) => {
            let mut out = IndexMap::new();
            out.insert("ok".to_string(), DolangValue::Bool(true));
            out.insert("status".to_string(), DolangValue::Int(0));
            out.insert("status_name".to_string(), DolangValue::Str("OK".to_string()));
            out.insert("body".to_string(), decode_message(&response.message)?);
            out.insert("metadata".to_string(), string_map_to_value(response.metadata));
            out.insert("error".to_string(), DolangValue::Null);
            Ok(DolangValue::Map(out))
        }
        Err(status) => {
            let mut error = IndexMap::new();
            error.insert(
                "code".to_string(),
                DolangValue::Int(i64::from(status.code() as i32)),
            );
            error.insert(
                "name".to_string(),
                DolangValue::Str(status.code().to_string()),
            );
            error.insert(
                "message".to_string(),
                DolangValue::Str(status.message().to_string()),
            );
            error.insert("details".to_string(), DolangValue::List(vec![]));

            let mut out = IndexMap::new();
            out.insert("ok".to_string(), DolangValue::Bool(false));
            out.insert(
                "status".to_string(),
                DolangValue::Int(i64::from(status.code() as i32)),
            );
            out.insert(
                "status_name".to_string(),
                DolangValue::Str(status.code().to_string()),
            );
            out.insert("body".to_string(), DolangValue::Null);
            out.insert(
                "metadata".to_string(),
                string_map_to_value(metadata_map_to_hash_map(status.metadata())),
            );
            out.insert("error".to_string(), DolangValue::Map(error));
            Ok(DolangValue::Map(out))
        }
    }
}

fn string_map_to_value(entries: std::collections::HashMap<String, String>) -> DolangValue {
    DolangValue::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key, DolangValue::Str(value)))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::call;
    use crate::interpreter::DolangValue;
    use crate::runtime::{RuntimeContext, RuntimeMode};
    use crate::stdlib_native::grpc_client::transport::tests::{fixture_root, start_echo_server};
    use crate::stdlib_native::grpc_client::types::GrpcContractKind;
    use indexmap::IndexMap;

    fn test_context() -> RuntimeContext {
        RuntimeContext::new(RuntimeMode::Test, fixture_root())
    }

    fn grpc_client() -> DolangValue {
        DolangValue::GrpcClient {
            target: "dns:///echo-service:50051".to_string(),
            contract_path: "tests/fixtures/grpc/descriptors/echo.pb".to_string(),
            contract_kind: GrpcContractKind::Descriptor,
            timeout_ms: Some(1500),
            metadata: IndexMap::new(),
        }
    }

    #[test]
    fn grpc_client_call_requires_config_map() {
        let context = test_context();
        let error = call(&grpc_client(), "call", &[], &context).expect_err("missing config should fail");
        assert!(error.to_string().contains("GrpcClient.call"));
    }

    #[test]
    fn grpc_client_rejects_unknown_method() {
        let context = test_context();
        let error = call(&grpc_client(), "missing", &[], &context).expect_err("unknown method should fail");
        assert!(error.to_string().contains("GrpcClient has no method"));
    }

    #[test]
    fn grpc_client_call_rejects_missing_service_from_descriptor() {
        let context = test_context();
        let error = call(
            &grpc_client(),
            "call",
            &[DolangValue::Map(IndexMap::from([
                (
                    "service".to_string(),
                    DolangValue::Str("dolang.test.echo.v1.MissingService".to_string()),
                ),
                ("method".to_string(), DolangValue::Str("Echo".to_string())),
                (
                    "body".to_string(),
                    DolangValue::Map(IndexMap::from([(
                        "message".to_string(),
                        DolangValue::Str("ping".to_string()),
                    )])),
                ),
            ]))],
            &context,
        )
        .expect_err("missing service should fail before transport");

        assert!(error.to_string().contains("MissingService"));
    }

    #[test]
    fn grpc_client_call_reports_connection_failure() {
        let context = test_context();
        let result = call(
            &grpc_client(),
            "call",
            &[DolangValue::Map(IndexMap::from([
                (
                    "service".to_string(),
                    DolangValue::Str("dolang.test.echo.v1.EchoService".to_string()),
                ),
                ("method".to_string(), DolangValue::Str("Echo".to_string())),
                (
                    "body".to_string(),
                    DolangValue::Map(IndexMap::from([(
                        "message".to_string(),
                        DolangValue::Str("ping".to_string()),
                    )])),
                ),
            ]))],
            &context,
        );

        let error = result.expect_err("unreachable target should fail to connect");
        assert!(error.to_string().contains("failed to connect"));
    }

    #[test]
    fn grpc_client_call_returns_structured_success_result() {
        let Some((target, shutdown, handle)) = start_echo_server() else {
            return;
        };
        let context = test_context();
        let client = DolangValue::GrpcClient {
            target,
            contract_path: "tests/fixtures/grpc/descriptors/echo.pb".to_string(),
            contract_kind: GrpcContractKind::Descriptor,
            timeout_ms: Some(2000),
            metadata: IndexMap::new(),
        };

        let result = call(
            &client,
            "call",
            &[DolangValue::Map(IndexMap::from([
                (
                    "service".to_string(),
                    DolangValue::Str("dolang.test.echo.v1.EchoService".to_string()),
                ),
                ("method".to_string(), DolangValue::Str("Echo".to_string())),
                (
                    "body".to_string(),
                    DolangValue::Map(IndexMap::from([(
                        "message".to_string(),
                        DolangValue::Str("ping".to_string()),
                    )])),
                ),
            ]))],
            &context,
        )
        .expect("live echo call should succeed");

        match result {
            DolangValue::Map(map) => {
                assert_eq!(map.get("ok"), Some(&DolangValue::Bool(true)));
                assert_eq!(map.get("status"), Some(&DolangValue::Int(0)));
                assert_eq!(
                    map.get("status_name"),
                    Some(&DolangValue::Str("OK".to_string()))
                );
                assert_eq!(
                    map.get("body"),
                    Some(&DolangValue::Map(IndexMap::from([(
                        "message".to_string(),
                        DolangValue::Str("ping".to_string()),
                    )])))
                );
                assert_eq!(map.get("error"), Some(&DolangValue::Null));
            }
            other => panic!("expected structured success map, got {}", other.type_name()),
        }

        let _ = shutdown.send(());
        handle.join().expect("server thread should exit cleanly");
    }
}
