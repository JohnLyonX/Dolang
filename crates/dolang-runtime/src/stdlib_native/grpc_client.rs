#[path = "grpc/config.rs"]
pub mod config;
#[path = "grpc/contract.rs"]
pub mod contract;
#[path = "grpc/mapper.rs"]
pub mod mapper;
#[path = "grpc/transport.rs"]
pub mod transport;
#[path = "grpc/types.rs"]
pub mod types;

use std::sync::Arc;

use crate::error::Error;
use crate::interpreter::DolangValue;
use crate::runtime::{NativeFnMap, RuntimeContext};
use config::parse_client_config;
use indexmap::IndexMap;

pub fn register(context: &mut RuntimeContext) {
    let mut exports = NativeFnMap::new();
    exports.insert(
        "client".into(),
        Arc::new(|args: &[DolangValue], _ctx: &RuntimeContext| client(args)),
    );

    context.register_native_module("std.grpc", exports);
}

fn client(args: &[DolangValue]) -> Result<DolangValue, Error> {
    let config = parse_client_config(args)?;
    Ok(DolangValue::GrpcClient {
        target: config.target,
        contract_path: config.contract_path,
        contract_kind: config.contract_kind,
        timeout_ms: config.timeout_ms,
        metadata: config
            .metadata
            .into_iter()
            .map(|(key, value)| (key, DolangValue::Str(value)))
            .collect::<IndexMap<_, _>>(),
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::register;
    use crate::interpreter::DolangValue;
    use crate::runtime::{RuntimeContext, RuntimeMode};
    use indexmap::IndexMap;

    fn test_context() -> RuntimeContext {
        RuntimeContext::new(RuntimeMode::Test, PathBuf::from("."))
    }

    pub(crate) fn grpc_client_args() -> Vec<DolangValue> {
        let mut config = IndexMap::new();
        config.insert(
            "target".to_string(),
            DolangValue::Str("dns:///users:50051".to_string()),
        );
        config.insert(
            "descriptor".to_string(),
            DolangValue::Str("descriptors/echo.pb".to_string()),
        );
        vec![DolangValue::Map(config)]
    }

    #[test]
    fn grpc_module_registers_client_export() {
        let mut context = test_context();
        register(&mut context);

        let module = context
            .native_module("std.grpc")
            .expect("std.grpc should be registered");
        assert!(module.get("client").is_some());
    }

    #[test]
    fn grpc_client_requires_target() {
        let mut context = test_context();
        register(&mut context);

        let module = context
            .native_module("std.grpc")
            .expect("std.grpc should be registered");
        let client = module
            .get("client")
            .expect("std.grpc should export client");
        let mut config = IndexMap::new();
        config.insert(
            "descriptor".to_string(),
            DolangValue::Str("descriptors/echo.pb".to_string()),
        );
        let error = client
            .as_ref()(&[DolangValue::Map(config)], &context)
            .expect_err("missing target should fail");

        assert!(error.to_string().contains("std.grpc.client"));
        assert!(error.to_string().contains("target"));
    }

    #[test]
    fn grpc_client_requires_descriptor_or_proto() {
        let mut context = test_context();
        register(&mut context);

        let module = context
            .native_module("std.grpc")
            .expect("std.grpc should be registered");
        let client = module
            .get("client")
            .expect("std.grpc should export client");
        let mut config = IndexMap::new();
        config.insert(
            "target".to_string(),
            DolangValue::Str("dns:///users:50051".to_string()),
        );
        let error = client
            .as_ref()(&[DolangValue::Map(config)], &context)
            .expect_err("missing descriptor/proto should fail");

        assert!(error.to_string().contains("descriptor"));
        assert!(error.to_string().contains("proto"));
    }

    #[test]
    fn grpc_client_returns_grpc_connection_handle() {
        let mut context = test_context();
        register(&mut context);

        let module = context
            .native_module("std.grpc")
            .expect("std.grpc should be registered");
        let client = module
            .get("client")
            .expect("std.grpc should export client");
        let result = client
            .as_ref()(&grpc_client_args(), &context)
            .expect("valid config should build a grpc client");

        match result {
            DolangValue::GrpcClient { target, .. } => assert_eq!(target, "dns:///users:50051"),
            other => panic!("expected grpc client handle, got {}", other.type_name()),
        }
    }
}
