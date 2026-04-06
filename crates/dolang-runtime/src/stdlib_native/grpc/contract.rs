use std::fs;
use std::path::Path;

use crate::error::Error;
use prost_reflect::{DescriptorPool, MethodDescriptor, ServiceDescriptor};

use super::types::{GrpcClientConfig, GrpcContractKind};

#[derive(Debug, Clone)]
pub struct GrpcContract {
    pool: DescriptorPool,
}

impl GrpcContract {
    pub fn load(config: &GrpcClientConfig, project_root: &Path) -> Result<Self, Error> {
        match config.contract_kind {
            GrpcContractKind::Descriptor => load_descriptor(project_root.join(&config.contract_path)),
            GrpcContractKind::Proto => Err(Error::Interpreter(
                "std.grpc.client(): proto loading is not implemented yet; use 'descriptor' for now"
                    .to_string(),
            )),
        }
    }

    pub fn service(&self, service_name: &str) -> Result<ServiceDescriptor, Error> {
        self.pool.get_service_by_name(service_name).ok_or_else(|| {
            Error::Interpreter(format!(
                "std.grpc: service '{}' was not found in the loaded descriptor set",
                service_name
            ))
        })
    }

    pub fn method(
        &self,
        service_name: &str,
        method_name: &str,
    ) -> Result<MethodDescriptor, Error> {
        let service = self.service(service_name)?;
        service
            .methods()
            .find(|method| method.name() == method_name)
            .ok_or_else(|| {
                Error::Interpreter(format!(
                    "std.grpc: method '{}.{}' was not found in the loaded descriptor set",
                    service_name, method_name
                ))
            })
    }
}

fn load_descriptor(path: impl AsRef<Path>) -> Result<GrpcContract, Error> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|err| {
        Error::Interpreter(format!(
            "std.grpc: failed to read descriptor '{}': {err}",
            path.display()
        ))
    })?;
    let pool = DescriptorPool::decode(bytes.as_slice()).map_err(|err| {
        Error::Interpreter(format!(
            "std.grpc: failed to decode descriptor '{}': {err}",
            path.display()
        ))
    })?;

    Ok(GrpcContract { pool })
}

#[cfg(test)]
mod tests {
    use super::GrpcContract;
    use crate::interpreter::DolangValue;
    use crate::runtime::{RuntimeContext, RuntimeMode};
    use crate::stdlib_native::grpc_client::config::parse_client_config;
    use std::path::PathBuf;

    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
    }

    fn descriptor_config() -> crate::stdlib_native::grpc_client::types::GrpcClientConfig {
        let args = [DolangValue::Map(indexmap::IndexMap::from([
            (
                "target".to_string(),
                DolangValue::Str("dns:///echo-service:50051".to_string()),
            ),
            (
                "descriptor".to_string(),
                DolangValue::Str("tests/fixtures/grpc/descriptors/echo.pb".to_string()),
            ),
        ]))];
        parse_client_config(&args).expect("descriptor config should parse")
    }

    #[test]
    fn loads_descriptor_and_finds_service_and_method() {
        let config = descriptor_config();
        let contract = GrpcContract::load(&config, &fixture_root()).expect("descriptor should load");

        let service = contract
            .service("dolang.test.echo.v1.EchoService")
            .expect("service should exist");
        let method = contract
            .method("dolang.test.echo.v1.EchoService", "Echo")
            .expect("method should exist");

        assert_eq!(service.name(), "EchoService");
        assert_eq!(method.name(), "Echo");
    }

    #[test]
    fn rejects_missing_descriptor_file() {
        let args = [DolangValue::Map(indexmap::IndexMap::from([
            (
                "target".to_string(),
                DolangValue::Str("dns:///echo-service:50051".to_string()),
            ),
            (
                "descriptor".to_string(),
                DolangValue::Str("tests/fixtures/grpc/descriptors/missing.pb".to_string()),
            ),
        ]))];
        let config = parse_client_config(&args).expect("config should parse");

        let error = GrpcContract::load(&config, &fixture_root())
            .expect_err("missing descriptor should fail");

        assert!(error.to_string().contains("failed to read descriptor"));
    }

    #[test]
    fn rejects_missing_service_and_method() {
        let config = descriptor_config();
        let contract = GrpcContract::load(&config, &fixture_root()).expect("descriptor should load");

        let service_error = contract
            .service("dolang.test.echo.v1.MissingService")
            .expect_err("missing service should fail");
        let method_error = contract
            .method("dolang.test.echo.v1.EchoService", "MissingMethod")
            .expect_err("missing method should fail");

        assert!(service_error.to_string().contains("MissingService"));
        assert!(method_error.to_string().contains("MissingMethod"));
    }

    #[test]
    fn proto_loading_returns_explicit_not_supported_error() {
        let args = [DolangValue::Map(indexmap::IndexMap::from([
            (
                "target".to_string(),
                DolangValue::Str("dns:///echo-service:50051".to_string()),
            ),
            (
                "proto".to_string(),
                DolangValue::Str("tests/fixtures/grpc/proto/echo.proto".to_string()),
            ),
        ]))];
        let config = parse_client_config(&args).expect("proto config should parse");
        let error = GrpcContract::load(&config, &fixture_root())
            .expect_err("proto path should return explicit error");

        assert!(error.to_string().contains("proto loading is not implemented yet"));
    }

    #[test]
    fn fixture_root_matches_runtime_project_layout() {
        let context = RuntimeContext::new(RuntimeMode::Test, fixture_root());
        assert!(context.project_root().join("tests/fixtures/grpc/proto/echo.proto").exists());
    }
}
