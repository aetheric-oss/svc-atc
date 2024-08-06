/// Types used for REST communication with the svc-cargo server
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use lib_common::time::{DateTime, Utc};

/// Example Request Body Information Type
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct AckRequest {
    /// Flight plan ID to confirm or reject
    pub fp_id: String,

    /// Acknowledgement Status
    pub status: AckStatus,
}

/// Confirm itinerary Operation Status
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema)]
pub enum AckStatus {
    /// Unauthorized request
    Deny,

    /// Unavailable Service
    Confirm,
}

/// Latitude, longitude, and altitude
///  following the WGS-84 standard
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema)]
pub struct Pose {
    /// Latitude
    pub latitude: f64,

    /// Longitude
    pub longitude: f64,

    /// Altitude in meters
    pub altitude_meters: f64,

    /// Heading with respect to true north
    pub heading_degrees: f64
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum MovementType {
    StartRotation, // rotate to new pose, then commence navigation
    EndRotation, // keep current pose, rotate at the end
    Smooth, // transition from current pose to end pose gradually
}

/// Information about cargo being carried
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Cargo {
    /// Cargo UUID
    pub id: String,

    // /// Cargo weight in grams
    // pub weight_g: u32
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum Phase {
    Takeoff(Pose),
    NavigateTo((Pose, MovementType)),
    PrecisionLand(Pose),
    BoxCapture(Pose), // combination precision landing, without touching down
    BoxRelease(Pose), // combination precision landing, without touching down
    CameraCapture(bool),
    PauseSeconds(u16), // differs from a 'hold' which freezes the current phase
}

/// Flight Plan Information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FlightPlan {
    /// Flight ID
    pub flight_uuid: String,

    /// Session ID
    pub session_id: String,

    /// Vehicle ID
    pub aircraft_id: String,

    /// Origin Vertiport ID
    pub origin_vertiport_id: String,

    /// Target Vertiport ID
    pub target_vertiport_id: String,

    /// Origin Vertipad ID
    pub origin_vertipad_id: String,

    /// Target Vertipad ID
    pub target_vertipad_id: String,

    /// Origin Time Start
    pub origin_timeslot_start: DateTime<Utc>,

    /// Origin Time End
    pub origin_timeslot_end: DateTime<Utc>,

    /// Target Time Start
    pub target_timeslot_start: DateTime<Utc>,

    /// Target Time End
    pub target_timeslot_end: DateTime<Utc>,

    /// Path
    pub phases: Vec<Phase>,

    /// Cargo to acquire
    pub acquire: Vec<Cargo>,

    /// Cargo to deliver
    pub deliver: Vec<Cargo>
}
