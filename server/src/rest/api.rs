//! Rest API implementations
/// openapi generated rest types
pub mod rest_types {
    include!("../../../openapi/types.rs");
}

pub use rest_types::*;

use crate::grpc::client::GrpcClients;
use axum::{body::Bytes, extract::Extension, Json};
use hyper::StatusCode;
use lib_common::time::{Duration, Utc};
use lib_common::uuid::to_uuid;
use std::fmt::{self, Display, Formatter};
use svc_storage_client_grpc::prelude::*;

// Provides a way to tell a caller if the service is healthy.
/// Checks dependencies, making sure all connections can be made.
#[utoipa::path(
    get,
    path = "/health",
    tag = "svc-atc",
    responses(
        (status = 200, description = "Service is healthy, all dependencies running."),
        (status = 503, description = "Service is unhealthy, one or more dependencies unavailable.")
    )
)]
pub async fn health_check(
    Extension(grpc_clients): Extension<GrpcClients>,
) -> Result<(), StatusCode> {
    rest_debug!("entry.");

    let mut ok = true;

    // This health check is to verify that ALL dependencies of this
    // microservice are running.
    if grpc_clients
        .storage
        .flight_plan
        .is_ready(ReadyRequest {})
        .await
        .is_err()
    {
        let error_msg = "svc-storage flight_plan unavailable.".to_string();
        rest_error!("{}.", &error_msg);
        ok = false;
    }

    match ok {
        true => {
            rest_debug!("healthy, all dependencies running.");
            Ok(())
        }
        false => {
            rest_error!("unhealthy, 1+ dependencies down.");
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

/// Errors in parsing flight plan data from storage
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FlightPlanError {
    /// Missing Data
    Data,

    /// Missing Origin Vertiport ID
    OriginVertiportId,

    /// Missing Target Vertiport ID
    TargetVertiportId,

    /// Missing Origin Timeslot Start
    OriginTimeslotStart,

    /// Missing Origin Timeslot End
    OriginTimeslotEnd,

    /// Missing Target Timeslot Start
    TargetTimeslotStart,

    /// Missing Target Timeslot End
    TargetTimeslotEnd,

    /// Missing Path
    Path,
}

impl Display for FlightPlanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FlightPlanError::Data => write!(f, "could not get data from object."),
            FlightPlanError::OriginVertiportId => {
                write!(f, "could not get origin_vertiport_id from data.")
            }
            FlightPlanError::TargetVertiportId => {
                write!(f, "could not get target_vertiport_id from data.")
            }
            FlightPlanError::OriginTimeslotStart => {
                write!(f, "could not get origin_timeslot_start from data.")
            }
            FlightPlanError::OriginTimeslotEnd => {
                write!(f, "could not get origin_timeslot_end from data.")
            }
            FlightPlanError::TargetTimeslotStart => {
                write!(f, "could not get target_timeslot_start from data.")
            }
            FlightPlanError::TargetTimeslotEnd => {
                write!(f, "could not get target_timeslot_end from data.")
            }
            FlightPlanError::Path => write!(f, "could not get path from data."),
        }
    }
}

impl TryFrom<flight_plan::Object> for FlightPlan {
    type Error = FlightPlanError;

    fn try_from(object: flight_plan::Object) -> Result<Self, Self::Error> {
        let flight_uuid = object.id;
        let data = object.data.ok_or_else(|| {
            rest_error!("could not get data from object.");
            FlightPlanError::Data
        })?;

        let origin_vertiport_id = data.origin_vertiport_id.ok_or_else(|| {
            rest_error!("could not get origin_vertiport_id from data.");
            FlightPlanError::OriginVertiportId
        })?;

        let target_vertiport_id = data.target_vertiport_id.ok_or_else(|| {
            rest_error!("could not get target_vertiport_id from data.");
            FlightPlanError::TargetVertiportId
        })?;

        let origin_timeslot_start = data.origin_timeslot_start.ok_or_else(|| {
            rest_error!("could not get origin_timeslot_start from data.");
            FlightPlanError::OriginTimeslotStart
        })?;

        let origin_timeslot_end = data.origin_timeslot_end.ok_or_else(|| {
            rest_error!("could not get origin_timeslot_end from data.");
            FlightPlanError::OriginTimeslotEnd
        })?;

        let target_timeslot_start = data.target_timeslot_start.ok_or_else(|| {
            rest_error!("could not get target_timeslot_start from data.");
            FlightPlanError::TargetTimeslotStart
        })?;

        let target_timeslot_end = data.target_timeslot_end.ok_or_else(|| {
            rest_error!("could not get target_timeslot_end from data.");
            FlightPlanError::TargetTimeslotEnd
        })?;

        let path = data.path.ok_or_else(|| {
            rest_error!("could not get path from data.");
            FlightPlanError::Path
        })?;

        let path = path
            .points
            .iter()
            .map(|p| PointZ {
                latitude: p.y,
                longitude: p.x,
                altitude_meters: p.z,
            })
            .collect();

        let plan = FlightPlan {
            session_id: data.session_id,
            flight_uuid,
            aircraft_id: data.vehicle_id,
            origin_vertiport_id,
            target_vertiport_id,
            origin_vertipad_id: data.origin_vertipad_id,
            target_vertipad_id: data.target_vertipad_id,
            origin_timeslot_start: origin_timeslot_start.into(),
            origin_timeslot_end: origin_timeslot_end.into(),
            target_timeslot_start: target_timeslot_start.into(),
            target_timeslot_end: target_timeslot_end.into(),
            path,
            acquire: vec![],
            deliver: vec![],
        };

        Ok(plan)
    }
}

/// Acknowledge a flight
#[utoipa::path(
    post,
    path = "/atc/acknowledge",
    tag = "svc-atc",
    request_body = AckRequest,
    responses(
        (status = 200, description = "Request successful.", body = String),
        (status = 500, description = "Request unsuccessful."),
    )
)]
pub async fn acknowledge_flight_plan(
    Extension(grpc_clients): Extension<GrpcClients>,
    Json(payload): Json<AckRequest>,
) -> Result<(), StatusCode> {
    rest_debug!("entry.");

    let id = to_uuid(&payload.fp_id).ok_or_else(|| {
        rest_error!("invalid flight plan UUID.");
        StatusCode::BAD_REQUEST
    })?;

    crate::common::ack_flight(id, &grpc_clients)
        .await
        .map_err(|e| {
            rest_error!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// Get flight plans
#[utoipa::path(
    get,
    path = "/atc/plans",
    tag = "svc-atc",
    request_body = String,
    responses(
        (status = 200, description = "Request successful.", body = String),
        (status = 500, description = "Request unsuccessful."),
    )
)]
pub async fn get_flight_plans(
    Extension(grpc_clients): Extension<GrpcClients>,
    aircraft_id: Bytes,
) -> Result<Json<Vec<FlightPlan>>, StatusCode> {
    rest_debug!("entry.");
    let aircraft_id = String::from_utf8(aircraft_id.to_vec()).map_err(|_| {
        rest_error!("could not convert aircraft_id to string.");
        StatusCode::BAD_REQUEST
    })?;

    let aircraft_id = to_uuid(&aircraft_id).ok_or_else(|| {
        rest_error!("invalid aircraft UUID.");
        StatusCode::BAD_REQUEST
    })?;

    let now = Utc::now();

    // TODO(R5): parameterize duration lookahead
    let delta = Duration::try_minutes(60).ok_or_else(|| {
        rest_error!("could not create duration.");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    //
    // TODO(R5): Move this search to a loop searching for all
    //  aircraft, and push results to local cache
    // So that we can return cached results and not search svc-storage
    //  every time an aircraft asks for flight plans
    //
    let filter =
        AdvancedSearchFilter::search_equals("vehicle_id".to_owned(), aircraft_id.to_string())
            .and_between(
                "origin_timeslot_start".to_owned(),
                (now - delta).to_string(),
                (now + delta).to_string(),
            );

    let mut plans = grpc_clients
        .storage
        .flight_plan
        .search(filter)
        .await
        .map_err(|e| {
            rest_error!("svc-storage failure: {e}");
            StatusCode::NOT_FOUND
        })?
        .into_inner()
        .list
        .into_iter()
        .filter_map(|object| FlightPlan::try_from(object).ok())
        .collect::<Vec<FlightPlan>>();

    // TODO(R6): Check cargo or rideshare
    for plan in plans.iter_mut() {
        let filter = AdvancedSearchFilter::search_equals(
            "flight_plan_id".to_owned(),
            plan.flight_uuid.clone(),
        );

        grpc_clients
            .storage
            .flight_plan_parcel
            .search(filter)
            .await
            .map_err(|e| {
                let error_msg = "svc-storage failure.".to_string();
                rest_error!("{}: {e}", &error_msg);

                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .into_inner()
            .list
            .into_iter()
            .for_each(|parcel| {
                let id = parcel.parcel_id;
                if parcel.acquire {
                    plan.acquire.push(Cargo {
                        id: id.clone(),
                        // weight_g: parcel.weight_g
                    })
                }

                if parcel.deliver {
                    plan.deliver.push(Cargo {
                        id,
                        // weight_g: parcel.weight_g
                    })
                }
            });
    }

    rest_debug!("returning {} plans.", plans.len());
    Ok(Json(plans))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_common::uuid::Uuid;
    use svc_storage_client_grpc::prelude::{GeoLineStringZ, GeoPointZ};

    #[test]
    fn test_from_flight_plan_object_valid() {
        let now = Utc::now();
        let object = flight_plan::Object {
            id: "id".to_string(),
            data: Some(flight_plan::Data {
                session_id: "session_id".to_string(),
                vehicle_id: "vehicle_id".to_string(),
                origin_vertiport_id: Some("origin_vertiport_id".to_string()),
                target_vertiport_id: Some("target_vertiport_id".to_string()),
                origin_vertipad_id: "origin_vertipad_id".to_string(),
                target_vertipad_id: "target_vertipad_id".to_string(),
                origin_timeslot_start: Some((now + Duration::try_seconds(1).unwrap()).into()),
                origin_timeslot_end: Some((now + Duration::try_seconds(2).unwrap()).into()),
                target_timeslot_start: Some((now + Duration::try_seconds(3).unwrap()).into()),
                target_timeslot_end: Some((now + Duration::try_seconds(4).unwrap()).into()),
                path: Some(GeoLineStringZ {
                    points: vec![GeoPointZ {
                        y: 1.0,
                        x: 2.0,
                        z: 3.0,
                    }],
                }),
                ..Default::default()
            }),
        };

        let flight_plan = FlightPlan::try_from(object).unwrap();
        assert_eq!(flight_plan.session_id, "session_id");
        assert_eq!(flight_plan.flight_uuid, "id");
        assert_eq!(flight_plan.aircraft_id, "vehicle_id");

        assert_eq!(
            flight_plan.origin_vertiport_id,
            "origin_vertiport_id".to_string()
        );
        assert_eq!(
            flight_plan.target_vertiport_id,
            "target_vertiport_id".to_string()
        );

        assert_eq!(flight_plan.origin_vertipad_id, "origin_vertipad_id");
        assert_eq!(flight_plan.target_vertipad_id, "target_vertipad_id");

        assert_eq!(
            flight_plan.origin_timeslot_start,
            now + Duration::try_seconds(1).unwrap()
        );
        assert_eq!(
            flight_plan.origin_timeslot_end,
            now + Duration::try_seconds(2).unwrap()
        );
        assert_eq!(
            flight_plan.target_timeslot_start,
            now + Duration::try_seconds(3).unwrap()
        );
        assert_eq!(
            flight_plan.target_timeslot_end,
            now + Duration::try_seconds(4).unwrap()
        );

        let path = flight_plan.path;
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].latitude, 1.0);
        assert_eq!(path[0].longitude, 2.0);
        assert_eq!(path[0].altitude_meters, 3.0);

        assert!(flight_plan.acquire.is_empty());
        assert!(flight_plan.deliver.is_empty());
    }

    #[test]
    fn test_from_flight_plan_object_invalid() {
        let now = Utc::now();
        let object = flight_plan::Object {
            id: "id".to_string(),
            data: Some(flight_plan::Data {
                session_id: "session_id".to_string(),
                vehicle_id: "vehicle_id".to_string(),
                origin_vertiport_id: Some("origin_vertiport_id".to_string()),
                target_vertiport_id: Some("target_vertiport_id".to_string()),
                origin_vertipad_id: "origin_vertipad_id".to_string(),
                target_vertipad_id: "target_vertipad_id".to_string(),
                origin_timeslot_start: Some((now + Duration::try_seconds(1).unwrap()).into()),
                origin_timeslot_end: Some((now + Duration::try_seconds(2).unwrap()).into()),
                target_timeslot_start: Some((now + Duration::try_seconds(3).unwrap()).into()),
                target_timeslot_end: Some((now + Duration::try_seconds(4).unwrap()).into()),
                path: Some(GeoLineStringZ {
                    points: vec![GeoPointZ {
                        y: 1.0,
                        x: 2.0,
                        z: 3.0,
                    }],
                }),
                ..Default::default()
            }),
        };

        let mut tmp = object.clone();
        tmp.data = None;
        assert_eq!(
            FlightPlan::try_from(tmp).unwrap_err(),
            FlightPlanError::Data
        );

        let mut tmp = object.clone();
        tmp.data.as_mut().unwrap().origin_vertiport_id = None;
        assert_eq!(
            FlightPlan::try_from(tmp).unwrap_err(),
            FlightPlanError::OriginVertiportId
        );

        let mut tmp = object.clone();
        tmp.data.as_mut().unwrap().target_vertiport_id = None;
        assert_eq!(
            FlightPlan::try_from(tmp).unwrap_err(),
            FlightPlanError::TargetVertiportId
        );

        let mut tmp = object.clone();
        tmp.data.as_mut().unwrap().origin_timeslot_start = None;
        assert_eq!(
            FlightPlan::try_from(tmp).unwrap_err(),
            FlightPlanError::OriginTimeslotStart
        );

        let mut tmp = object.clone();
        tmp.data.as_mut().unwrap().origin_timeslot_end = None;
        assert_eq!(
            FlightPlan::try_from(tmp).unwrap_err(),
            FlightPlanError::OriginTimeslotEnd
        );

        let mut tmp = object.clone();
        tmp.data.as_mut().unwrap().target_timeslot_start = None;
        assert_eq!(
            FlightPlan::try_from(tmp).unwrap_err(),
            FlightPlanError::TargetTimeslotStart
        );

        let mut tmp = object.clone();
        tmp.data.as_mut().unwrap().target_timeslot_end = None;
        assert_eq!(
            FlightPlan::try_from(tmp).unwrap_err(),
            FlightPlanError::TargetTimeslotEnd
        );

        let mut tmp = object.clone();
        tmp.data.as_mut().unwrap().path = None;
        assert_eq!(
            FlightPlan::try_from(tmp).unwrap_err(),
            FlightPlanError::Path
        );
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = crate::config::Config::default();
        let grpc_clients = GrpcClients::default(config);
        let result = health_check(Extension(grpc_clients)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_acknowledge_flight_plan() {
        // bad request - invalid uuid
        let payload = AckRequest {
            fp_id: "invalid".to_string(),
            status: AckStatus::Confirm,
        };
        let config = crate::config::Config::default();
        let grpc_clients = GrpcClients::default(config);
        let error = acknowledge_flight_plan(Extension(grpc_clients), Json(payload))
            .await
            .unwrap_err();
        assert_eq!(error, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_flight_plans() {
        let aircraft_id = Bytes::from("invalid");
        let config = crate::config::Config::default();
        let grpc_clients = GrpcClients::default(config);
        let error = get_flight_plans(Extension(grpc_clients.clone()), aircraft_id)
            .await
            .unwrap_err();
        assert_eq!(error, StatusCode::BAD_REQUEST);

        let aircraft_id = Bytes::from(vec![0xFF]); // non-UTF8 character
        let error = get_flight_plans(Extension(grpc_clients.clone()), aircraft_id)
            .await
            .unwrap_err();
        assert_eq!(error, StatusCode::BAD_REQUEST);

        let aircraft_id = Bytes::from(Uuid::new_v4().to_string());
        let results = get_flight_plans(Extension(grpc_clients.clone()), aircraft_id)
            .await
            .unwrap()
            .0;
        assert!(results.is_empty());

        let data = vehicle::mock::get_data_obj();
        let aircraft_id = grpc_clients
            .storage
            .vehicle
            .insert(data)
            .await
            .unwrap()
            .into_inner()
            .object
            .unwrap()
            .id;

        let results = get_flight_plans(
            Extension(grpc_clients.clone()),
            Bytes::from(aircraft_id.clone()),
        )
        .await
        .unwrap()
        .0;

        assert!(results.is_empty());
    }

    #[test]
    fn test_flight_plan_error_display() {
        assert_eq!(
            FlightPlanError::Data.to_string(),
            "could not get data from object."
        );
        assert_eq!(
            FlightPlanError::OriginVertiportId.to_string(),
            "could not get origin_vertiport_id from data."
        );
        assert_eq!(
            FlightPlanError::TargetVertiportId.to_string(),
            "could not get target_vertiport_id from data."
        );
        assert_eq!(
            FlightPlanError::OriginTimeslotStart.to_string(),
            "could not get origin_timeslot_start from data."
        );
        assert_eq!(
            FlightPlanError::OriginTimeslotEnd.to_string(),
            "could not get origin_timeslot_end from data."
        );
        assert_eq!(
            FlightPlanError::TargetTimeslotStart.to_string(),
            "could not get target_timeslot_start from data."
        );
        assert_eq!(
            FlightPlanError::TargetTimeslotEnd.to_string(),
            "could not get target_timeslot_end from data."
        );
        assert_eq!(
            FlightPlanError::Path.to_string(),
            "could not get path from data."
        );
    }
}
