
// A cargo itinerary can be a single flight or a series of flights.
// If the aircraft must land and power off while cargo is loaded, the itinerary
//   will contain multiple flights.
// If the aircraft autonomously picks up cargo or lowers a winch for a payload, the itinerary
//   will contain a single flight because the aircraft will not land or power off between
//   legs.

pub enum PickupType {
    Winch,
    Swoop,
    Land
}

/// Latitude, longitude, and altitude
///  following the WGS-84 standard
// #[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema)]
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

pub struct HopData {
    pub takeoff_location: Pose,
    pub takeoff_time: DateTime<Utc>,
    pub landing_location: Pose,
    pub landing_time: DateTime<Utc>,
    pub waypoints: Vec<Pose>,
    pub cruise_speed: u32,
    pub hover_speed: u32,
    pub autopilot_type: mavlink::common::MAV_AUTOPILOT,
    pub vehicle_type: mavlink::common::MAV_TYPE,
    pub gimbal_camera_id: u8
}

pub struct ContinuousData {
    pub takeoff_location: Pose,
    pub landing_location: Pose,
    pub pickup_location: Pose,
    pub dropoff_location: Pose,
    pub takeoff_time: DateTime<Utc>,
    pub landing_time: DateTime<Utc>,
    pub pickup_time: DateTime<Utc>,
    pub dropoff_time: DateTime<Utc>,
    pub waypoints_deadhead_a: Vec<Pose>,
    pub waypoints_deadhead_b: Vec<Pose>,
    pub waypoints_main: Vec<Pose>,
    pub cruise_speed: u32,
    pub hover_speed: u32,
    pub autopilot_type: mavlink::common::MAV_AUTOPILOT,
    pub vehicle_type: mavlink::common::MAV_TYPE,
    pub gentle_dropoff: bool,
    pub gimbal_camera_id: u8
}

trait Specification {
    fn try_hop(data: Hop) -> Result<Self, ()>;
    fn try_winch(data: Winch) -> Result<Self, ()>;
    fn try_swoop(data: Swoop) -> Result<Self, ()>;
}

