use std::collections::HashMap;
use std::time::Duration;

use prost::Message;
use prost_reflect::{DynamicMessage, MessageDescriptor};
use tonic::codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};
use tonic::metadata::{AsciiMetadataKey, MetadataMap, MetadataValue};
use tonic::transport::Endpoint;
use tonic::{Request, Status, client::Grpc};
use tonic::codegen::http::uri::PathAndQuery;

use crate::error::Error;

use super::types::{GrpcCallConfig, GrpcClientConfig};

pub struct UnarySuccess {
    pub message: DynamicMessage,
    pub metadata: HashMap<String, String>,
}

pub fn execute_unary(
    client: &GrpcClientConfig,
    call: &GrpcCallConfig,
    method: &prost_reflect::MethodDescriptor,
    request: DynamicMessage,
) -> Result<Result<UnarySuccess, Status>, Error> {
    let client = client.clone();
    let call = call.clone();
    let method = method.clone();

    // When called from within a tokio runtime (e.g. axum serve mode), reuse it
    // via block_in_place so tower's Buffer background worker can run on other
    // threads. Without this, a single-threaded runtime cannot release the Buffer
    // semaphore, causing "buffer full; poll_ready must be called first" panics.
    //
    // When no runtime is present (e.g. script mode), create a small multi-thread
    // runtime so background tasks still get scheduled.
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(move || {
            handle.block_on(execute_unary_async(client, call, method, request))
        }),
        Err(_) => {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|err| {
                    Error::Interpreter(format!("std.grpc: failed to build runtime: {err}"))
                })?;
            runtime.block_on(execute_unary_async(client, call, method, request))
        }
    }
}

async fn execute_unary_async(
    client: GrpcClientConfig,
    call: GrpcCallConfig,
    method: prost_reflect::MethodDescriptor,
    request_message: DynamicMessage,
) -> Result<Result<UnarySuccess, Status>, Error> {
    let endpoint = endpoint_from_target(&client.target, call.timeout_ms.or(client.timeout_ms))?;
    let channel = endpoint
        .connect()
        .await
        .map_err(|err| Error::Interpreter(format!("std.grpc: failed to connect: {err}")))?;

    let path = format!("/{}/{}", call.service, call.method)
        .parse::<PathAndQuery>()
        .map_err(|err| Error::Interpreter(format!("std.grpc: invalid method path: {err}")))?;

    let mut grpc = Grpc::new(channel);

    // tonic 0.12.3's Grpc::streaming() calls self.inner.call() directly without
    // calling poll_ready() first, violating the tower service contract and causing
    // tower::Buffer to panic. We must explicitly call ready() to acquire the
    // semaphore permit before unary().
    grpc.ready()
        .await
        .map_err(|err| Error::Interpreter(format!("std.grpc: channel not ready: {err}")))?;

    let mut request = Request::new(request_message);
    for (key, value) in client.metadata.iter().chain(call.metadata.iter()) {
        let metadata_key = key.parse::<AsciiMetadataKey>().map_err(|err| {
            Error::Interpreter(format!("std.grpc: invalid metadata key '{}': {err}", key))
        })?;
        let metadata_value = value.parse::<MetadataValue<_>>().map_err(|err| {
            Error::Interpreter(format!(
                "std.grpc: invalid metadata value for '{}': {err}",
                key
            ))
        })?;
        request.metadata_mut().insert(metadata_key, metadata_value);
    }
    if let Some(timeout_ms) = call.timeout_ms.or(client.timeout_ms) {
        request.set_timeout(Duration::from_millis(timeout_ms));
    }

    let codec = DynamicCodec::new(method.input(), method.output());
    match grpc.unary(request, path, codec).await {
        Ok(response) => {
            let metadata = metadata_map_to_hash_map(response.metadata());
            let message = response.into_inner();
            Ok(Ok(UnarySuccess { message, metadata }))
        }
        Err(status) => Ok(Err(status)),
    }
}

fn endpoint_from_target(target: &str, timeout_ms: Option<u64>) -> Result<Endpoint, Error> {
    let normalized = if target.starts_with("http://") || target.starts_with("https://") {
        target.to_string()
    } else if let Some(rest) = target.strip_prefix("dns:///") {
        format!("http://{rest}")
    } else {
        format!("http://{target}")
    };

    let endpoint = Endpoint::from_shared(normalized)
        .map_err(|err| Error::Interpreter(format!("std.grpc: invalid target endpoint: {err}")))?;

    Ok(match timeout_ms {
        Some(timeout_ms) => endpoint.timeout(Duration::from_millis(timeout_ms)),
        None => endpoint,
    })
}

pub fn metadata_map_to_hash_map(metadata: &MetadataMap) -> HashMap<String, String> {
    metadata
        .iter()
        .filter_map(|entry| match entry {
            tonic::metadata::KeyAndValueRef::Ascii(key, value) => {
                Some((key.as_str().to_string(), value.to_str().ok()?.to_string()))
            }
            tonic::metadata::KeyAndValueRef::Binary(_, _) => None,
        })
        .collect()
}

#[derive(Clone)]
struct DynamicCodec {
    encoder_desc: MessageDescriptor,
    decoder_desc: MessageDescriptor,
}

impl DynamicCodec {
    fn new(encoder_desc: MessageDescriptor, decoder_desc: MessageDescriptor) -> Self {
        Self {
            encoder_desc,
            decoder_desc,
        }
    }
}

impl Codec for DynamicCodec {
    type Encode = DynamicMessage;
    type Decode = DynamicMessage;
    type Encoder = DynamicEncoder;
    type Decoder = DynamicDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        DynamicEncoder {
            _descriptor: self.encoder_desc.clone(),
        }
    }

    fn decoder(&mut self) -> Self::Decoder {
        DynamicDecoder {
            descriptor: self.decoder_desc.clone(),
        }
    }
}

#[derive(Clone)]
struct DynamicEncoder {
    _descriptor: MessageDescriptor,
}

impl Encoder for DynamicEncoder {
    type Item = DynamicMessage;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        item.encode(dst)
            .map_err(|err| Status::internal(format!("dynamic encode failed: {err}")))
    }
}

#[derive(Clone)]
struct DynamicDecoder {
    descriptor: MessageDescriptor,
}

impl Decoder for DynamicDecoder {
    type Item = DynamicMessage;
    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        DynamicMessage::decode(self.descriptor.clone(), src)
            .map(Some)
            .map_err(|err| Status::internal(format!("dynamic decode failed: {err}")))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{execute_unary, endpoint_from_target};
    use crate::interpreter::DolangValue;
    use crate::stdlib_native::grpc_client::config::parse_client_config;
    use crate::stdlib_native::grpc_client::contract::GrpcContract;
    use crate::stdlib_native::grpc_client::mapper::{decode_message, encode_message};
    use crate::stdlib_native::grpc_client::types::GrpcCallConfig;
    use indexmap::IndexMap;
    use prost_reflect::{DynamicMessage, MessageDescriptor, Value};
    use std::convert::Infallible;
    use std::future::{Ready, ready};
    use std::net::TcpListener as StdTcpListener;
    use std::thread;
    use tonic::body::{BoxBody, empty_body};
    use tonic::codegen::{BoxFuture, Service, http};
    use tonic::server::{Grpc, NamedService};
    use tonic::transport::{Server, server::TcpIncoming};
    use tonic::{Request, Response, Status};

    #[test]
    fn endpoint_normalizes_dns_style_target() {
        let endpoint = endpoint_from_target("dns:///127.0.0.1:50051", Some(1500))
            .expect("dns style target should normalize");
        assert_eq!(endpoint.uri().to_string(), "http://127.0.0.1:50051/");
    }

    #[test]
    fn execute_unary_round_trips_against_live_echo_server() {
        let Some((target, shutdown, handle)) = start_echo_server() else {
            return;
        };
        let client = parse_client_config(&[DolangValue::Map(IndexMap::from([
            ("target".to_string(), DolangValue::Str(target)),
            (
                "descriptor".to_string(),
                DolangValue::Str("tests/fixtures/grpc/descriptors/echo.pb".to_string()),
            ),
        ]))])
        .expect("client config should parse");
        let contract = GrpcContract::load(&client, &fixture_root()).expect("contract should load");
        let method = contract
            .method("dolang.test.echo.v1.EchoService", "Echo")
            .expect("echo method should exist");
        let request = encode_message(
            method.input(),
            &DolangValue::Map(IndexMap::from([(
                "message".to_string(),
                DolangValue::Str("hello live grpc".to_string()),
            )])),
        )
        .expect("request should encode");
        let call = GrpcCallConfig {
            service: "dolang.test.echo.v1.EchoService".to_string(),
            method: "Echo".to_string(),
            body: DolangValue::Map(IndexMap::from([(
                "message".to_string(),
                DolangValue::Str("hello live grpc".to_string()),
            )])),
            timeout_ms: Some(2000),
            metadata: std::collections::HashMap::new(),
        };

        let response = execute_unary(&client, &call, &method, request)
            .expect("transport should not fail locally")
            .expect("grpc status should be ok");
        let value = decode_message(&response.message).expect("response should decode");

        assert_eq!(
            value,
            DolangValue::Map(IndexMap::from([(
                "message".to_string(),
                DolangValue::Str("hello live grpc".to_string()),
            )]))
        );

        let _ = shutdown.send(());
        handle.join().expect("server thread should exit cleanly");
    }

    pub(crate) fn fixture_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
    }

    pub(crate) fn start_echo_server() -> Option<(String, tokio::sync::oneshot::Sender<()>, thread::JoinHandle<()>)> {
        let std_listener = match StdTcpListener::bind("127.0.0.1:0") {
            Ok(listener) => listener,
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => return None,
            Err(err) => panic!("ephemeral tcp listener should bind: {err}"),
        };
        let addr = std_listener
            .local_addr()
            .expect("listener should report local addr");
        std_listener
            .set_nonblocking(true)
            .expect("listener should allow nonblocking mode");
        let listener = tokio::net::TcpListener::from_std(std_listener)
            .expect("tokio listener should wrap std listener");
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let handle = thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("server runtime should build");
            runtime.block_on(async move {
                let incoming =
                    TcpIncoming::from_listener(listener, true, None).expect("incoming should bind");
                Server::builder()
                    .add_service(EchoGrpcService::new())
                    .serve_with_incoming_shutdown(incoming, async {
                        let _ = shutdown_rx.await;
                    })
                    .await
                    .expect("echo grpc server should serve");
            });
        });

        Some((format!("http://{addr}"), shutdown_tx, handle))
    }

    #[derive(Clone)]
    struct EchoGrpcService {
        input_desc: MessageDescriptor,
        output_desc: MessageDescriptor,
    }

    impl EchoGrpcService {
        fn new() -> Self {
            let config = parse_client_config(&[DolangValue::Map(IndexMap::from([
                (
                    "target".to_string(),
                    DolangValue::Str("http://127.0.0.1:1".to_string()),
                ),
                (
                    "descriptor".to_string(),
                    DolangValue::Str("tests/fixtures/grpc/descriptors/echo.pb".to_string()),
                ),
            ]))])
            .expect("server-side config should parse");
            let contract = GrpcContract::load(&config, &fixture_root()).expect("contract should load");
            let method = contract
                .method("dolang.test.echo.v1.EchoService", "Echo")
                .expect("echo method should exist");

            Self {
                input_desc: method.input(),
                output_desc: method.output(),
            }
        }
    }

    impl NamedService for EchoGrpcService {
        const NAME: &'static str = "dolang.test.echo.v1.EchoService";
    }

    impl Service<http::Request<BoxBody>> for EchoGrpcService {
        type Response = http::Response<BoxBody>;
        type Error = Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;

        fn poll_ready(
            &mut self,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: http::Request<BoxBody>) -> Self::Future {
            let input_desc = self.input_desc.clone();
            let output_desc = self.output_desc.clone();
            Box::pin(async move {
                if req.uri().path() != "/dolang.test.echo.v1.EchoService/Echo" {
                    return Ok(http::Response::builder()
                        .status(404)
                        .body(empty_body())
                        .expect("404 response should build"));
                }

                let codec = super::DynamicCodec::new(input_desc, output_desc.clone());
                let mut grpc = Grpc::new(codec);
                Ok(grpc.unary(EchoUnaryService { output_desc }, req).await)
            })
        }
    }

    #[derive(Clone)]
    struct EchoUnaryService {
        output_desc: MessageDescriptor,
    }

    impl Service<Request<DynamicMessage>> for EchoUnaryService {
        type Response = Response<DynamicMessage>;
        type Error = Status;
        type Future = Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(
            &mut self,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }

        fn call(&mut self, request: Request<DynamicMessage>) -> Self::Future {
            let input = request.into_inner();
            let message = match input.get_field_by_name("message") {
                Some(value) => match value.as_ref() {
                    Value::String(value) => value.clone(),
                    _ => String::new(),
                },
                None => String::new(),
            };
            let output = encode_message(
                self.output_desc.clone(),
                &DolangValue::Map(IndexMap::from([(
                    "message".to_string(),
                    DolangValue::Str(message),
                )])),
            )
            .expect("response message should encode");

            ready(Ok(Response::new(output)))
        }
    }
}
