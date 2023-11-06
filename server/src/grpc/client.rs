//! gRPC client helpers implementation
use tokio::sync::OnceCell;

use svc_storage_client_grpc::prelude::Clients;

pub(crate) static CLIENTS: OnceCell<GrpcClients> = OnceCell::const_new();

/// Returns CLIENTS, a GrpcClients object with default values.
/// Uses host and port configurations using a Config object generated from
/// environment variables.
/// Initializes CLIENTS if it hasn't been initialized yet.
pub async fn get_clients() -> &'static GrpcClients {
    CLIENTS
        .get_or_init(|| async move {
            let config = crate::Config::try_from_env().unwrap_or_default();
            GrpcClients::default(config)
        })
        .await
}

/// Struct to hold all gRPC client connections
#[derive(Clone, Debug)]
pub struct GrpcClients {
    /// Svc-Storage clients
    pub storage: Clients,
}

impl GrpcClients {
    /// Create new GrpcClients with defaults
    pub fn default(config: crate::config::Config) -> Self {
        let storage_clients = Clients::new(config.storage_host_grpc, config.storage_port_grpc);

        GrpcClients {
            storage: storage_clients,
        }
    }
}

#[cfg(test)]
mod tests {
    use svc_storage_client_grpc::prelude::Client;

    use super::*;

    #[tokio::test]
    async fn test_grpc_clients_default() {
        crate::get_log_handle().await;
        ut_info!("(test_grpc_clients_default) Start.");

        let config = crate::Config::default();
        let clients = GrpcClients::default(config);

        let flight_plan = &clients.storage.flight_plan;
        ut_debug!("(test_grpc_clients_default) flight_plan: {:?}", flight_plan);
        assert_eq!(flight_plan.get_name(), "flight_plan");

        ut_info!("(test_grpc_clients_default) Success.");
    }
}
