//! gRPC client helpers implementation

pub use svc_storage_client_grpc::Clients;

/// Struct to hold all gRPC client connections
#[derive(Clone, Debug)]
#[allow(missing_copy_implementations)]
pub struct GrpcClients {
    /// Storage microservice gRPC client
    pub storage: Clients,
}

impl GrpcClients {
    /// Creates default clients
    pub fn new(config: crate::config::Config) -> Self {
        GrpcClients {
            storage: Clients::new(config.storage_host_grpc, config.storage_port_grpc),
        }
    }
}
