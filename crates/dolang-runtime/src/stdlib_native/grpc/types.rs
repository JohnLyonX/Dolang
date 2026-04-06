#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrpcClientConfig {
    pub target: String,
    pub contract_path: String,
    pub contract_kind: GrpcContractKind,
    pub timeout_ms: Option<u64>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrpcContractKind {
    Descriptor,
    Proto,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrpcCallConfig {
    pub service: String,
    pub method: String,
    pub body: crate::interpreter::DolangValue,
    pub timeout_ms: Option<u64>,
    pub metadata: std::collections::HashMap<String, String>,
}
