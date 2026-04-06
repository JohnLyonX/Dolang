use dolang::DolangValue;
use dolang::stdlib_native::grpc_client::config::parse_client_config;
use dolang::stdlib_native::grpc_client::contract::GrpcContract;
use dolang::stdlib_native::grpc_client::mapper::encode_message;
use indexmap::IndexMap;
use prost_reflect::{DynamicMessage, MessageDescriptor, Value};
use std::convert::Infallible;
use std::future::ready;
use std::net::TcpListener as StdTcpListener;
use std::thread;
use tonic::body::{BoxBody, empty_body};
use tonic::codegen::{BoxFuture, Service, http};
use tonic::server::{Grpc, NamedService};
use tonic::transport::{Server, server::TcpIncoming};
use tonic::metadata::MetadataMap;
use tonic::{Code, Request, Response, Status};

pub fn fixture_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("crates")
        .join("dolang-runtime")
        .join("..")
        .join("..")
}

pub fn start_echo_server() -> Option<(String, tokio::sync::oneshot::Sender<()>, thread::JoinHandle<()>)> {
    start_server(ResponseMode::Echo)
}

pub fn start_not_found_server(
) -> Option<(String, tokio::sync::oneshot::Sender<()>, thread::JoinHandle<()>)> {
    start_server(ResponseMode::NotFound)
}

pub fn start_slow_server() -> Option<(String, tokio::sync::oneshot::Sender<()>, thread::JoinHandle<()>)> {
    start_server(ResponseMode::Slow)
}

pub fn start_unavailable_server(
) -> Option<(String, tokio::sync::oneshot::Sender<()>, thread::JoinHandle<()>)> {
    start_server(ResponseMode::Unavailable)
}

fn start_server(
    mode: ResponseMode,
) -> Option<(String, tokio::sync::oneshot::Sender<()>, thread::JoinHandle<()>)> {
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
                .add_service(EchoGrpcService::new(mode))
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
    mode: ResponseMode,
}

impl EchoGrpcService {
    fn new(mode: ResponseMode) -> Self {
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
            mode,
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
        let mode = self.mode.clone();
        Box::pin(async move {
            if req.uri().path() != "/dolang.test.echo.v1.EchoService/Echo" {
                return Ok(http::Response::builder()
                    .status(404)
                    .body(empty_body())
                    .expect("404 response should build"));
            }

            let codec = DynamicCodec::new(input_desc, output_desc.clone());
            let mut grpc = Grpc::new(codec);
            Ok(grpc.unary(EchoUnaryService { output_desc, mode }, req).await)
        })
    }
}

#[derive(Clone)]
struct EchoUnaryService {
    output_desc: MessageDescriptor,
    mode: ResponseMode,
}

impl Service<Request<DynamicMessage>> for EchoUnaryService {
    type Response = Response<DynamicMessage>;
    type Error = Status;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request<DynamicMessage>) -> Self::Future {
        if matches!(self.mode, ResponseMode::NotFound) {
            let mut metadata = MetadataMap::new();
            metadata.insert("x-grpc-error-source", "echo-backend".parse().expect("metadata"));
            return Box::pin(ready(Err(Status::with_metadata(
                Code::NotFound,
                "echo resource not found",
                metadata,
            ))));
        }
        if matches!(self.mode, ResponseMode::Slow) {
            let output_desc = self.output_desc.clone();
            return Box::pin(async move {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                let output = encode_message(
                    output_desc,
                    &DolangValue::Map(IndexMap::from([(
                        "message".to_string(),
                        DolangValue::Str("slow".to_string()),
                    )])),
                )
                .expect("slow response should encode");
                let mut response = Response::new(output);
                response.metadata_mut().insert(
                    "x-grpc-test",
                    "slow".parse().expect("response metadata"),
                );
                Ok(response)
            });
        }
        if matches!(self.mode, ResponseMode::Unavailable) {
            let mut metadata = MetadataMap::new();
            metadata.insert("x-grpc-error-source", "echo-backend".parse().expect("metadata"));
            return Box::pin(ready(Err(Status::with_metadata(
                Code::Unavailable,
                "echo backend unavailable",
                metadata,
            ))));
        }

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

        let mut response = Response::new(output);
        response.metadata_mut().insert(
            "x-grpc-test",
            "echo-ok".parse().expect("response metadata"),
        );
        Box::pin(ready(Ok(response)))
    }
}

#[derive(Clone)]
enum ResponseMode {
    Echo,
    NotFound,
    Slow,
    Unavailable,
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

impl tonic::codec::Codec for DynamicCodec {
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

impl tonic::codec::Encoder for DynamicEncoder {
    type Item = DynamicMessage;
    type Error = Status;

    fn encode(
        &mut self,
        item: Self::Item,
        dst: &mut tonic::codec::EncodeBuf<'_>,
    ) -> Result<(), Self::Error> {
        prost::Message::encode(&item, dst)
            .map_err(|err| Status::internal(format!("dynamic encode failed: {err}")))
    }
}

#[derive(Clone)]
struct DynamicDecoder {
    descriptor: MessageDescriptor,
}

impl tonic::codec::Decoder for DynamicDecoder {
    type Item = DynamicMessage;
    type Error = Status;

    fn decode(
        &mut self,
        src: &mut tonic::codec::DecodeBuf<'_>,
    ) -> Result<Option<Self::Item>, Self::Error> {
        DynamicMessage::decode(self.descriptor.clone(), src)
            .map(Some)
            .map_err(|err| Status::internal(format!("dynamic decode failed: {err}")))
    }
}
