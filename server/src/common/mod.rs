//! Common code for GRPC and REST servers

#[macro_use]
pub mod macros;

use crate::grpc::client::GrpcClients;
use lib_common::time::Utc;
use lib_common::uuid::Uuid;
use std::fmt;
use svc_storage_client_grpc::prelude::*;

/// Error type for ack_flight
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AckError {
    /// Internal Error
    Internal,

    /// Dependencies not available
    Unavailable,

    /// Flight Plan Not Found
    NotFound,
}

impl fmt::Display for AckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AckError::Internal => write!(f, "Internal Error"),
            AckError::Unavailable => write!(f, "Dependencies not available"),
            AckError::NotFound => write!(f, "Flight Plan Not Found"),
        }
    }
}

/// This request might come in over REST, or through GRPC someday
///  if another microservice has a software-hardware link to a radio antenna
pub async fn ack_flight(fp_id: Uuid, grpc_clients: &GrpcClients) -> Result<(), AckError> {
    let mut data = grpc_clients
        .storage
        .flight_plan
        .get_by_id(Id {
            id: fp_id.to_string(),
        })
        .await
        .map_err(|e| {
            common_error!("(ack_flight) {}", e);
            AckError::NotFound
        })?
        .into_inner()
        .data
        .ok_or_else(|| {
            common_error!("(ack_flight) Couldn't get data from object id: {}", fp_id);
            AckError::Internal
        })?;

    data.carrier_ack = Some(Utc::now().into());

    //
    // Update the flight plan record to show that it has been acknowledged
    //
    let request = flight_plan::UpdateObject {
        id: fp_id.to_string(),
        data: Some(data),
        mask: Some(FieldMask {
            paths: vec!["carrier_ack".to_string()],
        }),
    };

    grpc_clients
        .storage
        .flight_plan
        .update(request)
        .await
        .map_err(|e| {
            common_error!("(ack_flight) {}", e);
            AckError::Internal
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ack_flight() {
        let fp_id = Uuid::new_v4();
        let config = crate::config::Config::default();
        let grpc_clients = GrpcClients::default(config);
        let error = ack_flight(fp_id, &grpc_clients).await.unwrap_err();
        assert_eq!(error, AckError::NotFound);

        let data = flight_plan::mock::get_data_obj();
        let fp_id = grpc_clients
            .storage
            .flight_plan
            .insert(data)
            .await
            .unwrap()
            .into_inner()
            .object
            .unwrap()
            .id;

        let fp_id = Uuid::parse_str(&fp_id).unwrap();
        let _ = ack_flight(fp_id, &grpc_clients).await.unwrap();
    }
}
