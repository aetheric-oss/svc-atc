use serde::{Serialize, Deserialize};
use lib_common::time::{DateTime, Utc};
use mavlink::common::MavCmd;
use num_traits::ToPrimitive;

const GEOFENCE_VERSION: u8 = 2;
const RALLYPOINTS_VERSION: u8 = 2;
const MISSION_VERSION: u8 = 2;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    #[serde(rename = "type")]
    type_: String,
    auto_continue: bool,

    command: u16,
    do_jump_id: u16,
    frame: u8,
    params: Vec<f64>,

    #[serde(rename = "Altitude")]
    altitude: f32,

    #[serde(rename = "AltitudeMode")]
    altitude_mode: u8,

    #[serde(rename = "AMSLAltAboveTerrain")]
    amsl_alt_above_terrain: f32,
}

impl Default for Item {
    fn default() -> Self {
        Item {
            type_: "SimpleItem".to_string(),
            auto_continue: true,
            command: MavCmd::MAV_CMD_LOGGING_START as u16,
            do_jump_id: 0,
            frame: mavlink::common::MAV_FRAME::MAV_FRAME_GLOBAL as u8,
            params: vec![
                0.0, // ULOG
                "null",
                "null",
                "null",
                "null",
                "null",
                "null",
            ],
            altitude: 0.0,
            altitude_mode: 1,
            amsl_alt_above_terrain: 0.0,
        }
    }
}

impl Item {
    fn landing(pose: Pose, yaw: f64, pitch: f64) -> Self {
        Item {
            command: MavCmd::MAV_CMD_NAV_LAND as u16,
            params: vec![
                pitch, // degrees
                "null",
                "null",
                yaw, // degrees
                pose.latitude, // latitude
                pose.longitude, // longitude
                pose.altitude, // altitude m
            ],
            ..Item::default()
        }
    }

    fn takeoff(target_pose: Pose, yaw: f64, pitch: f64) -> Self {
        Item {
            command: MavCmd::MAV_CMD_NAV_TAKEOFF as u16,
            frame: mavlink::common::MAV_FRAME::MAV_FRAME_GLOBAL as u8,
            params: vec![
                pitch, // degrees
                "null",
                "null",
                yaw, // degrees
                target_pose.latitude, // latitude
                target_pose.longitude, // longitude
                target_pose.altitude, // altitude m
            ],
            ..Item::default()
        }
    }

    fn delay(time: DateTime<Utc>) -> Self {
        Item {
            command: MavCmd::MAV_CMD_NAV_DELAY as u16,
            frame: mavlink::common::MAV_FRAME::MAV_FRAME_GLOBAL_RELATIVE_ALT as u8,
            altitude: 0.0,
            params: vec![
                -1, // enable time-of-day (the following) fields
                time.minute(), // minute
                time.second(), // second
                time.hour(), // hour
                "null",
                "null",
                "null",
            ],
            ..Item::default()
        }
    }

    fn waypoint(pose: Pose) -> Self {
        Item {
            command: MavCmd::MAV_CMD_NAV_WAYPOINT as u16,
            params: vec![
                0.0, // hold time
                0.0, // acceptance radius
                0.0, // pass-thru
                "NaN", // use current system yaw heading mode
                pose.latitude, // latitude
                pose.longitude, // longitude
                pose.altitude, // altitude m
            ],
            ..Item::default()
        }
    }

    fn try_transition(target_state: mavlink::common::MAV_VTOL_STATE) -> Result<Self, ()> {
        match target_state {
            MAV_VTOL_STATE::MAV_VTOL_STATE_FW | MAV_VTOL_STATE::MAV_VTOL_STATE_MC => {},
            _ => {
                common_error!("Invalid target state provided.");
                return Err(())
            }
        }

        let item = Item {
            command: MavCmd::MAV_CMD_DO_VTOL_TRANSITION as u16,
            frame: mavlink::common::MAV_FRAME::MAV_FRAME_GLOBAL_RELATIVE_ALT as u8,
            altitude: 0.0,
            params: vec![
                transition as u8,
                0, // normal transition
                "null",
                "null",
                "null",
                "null",
                "null",
            ],
            ..Item::default()
        }

        Ok(item)
    }

    fn gripper(action: mavlink::common::GRIPPER_ACTIONS) -> Self {
        Item {
            command: MavCmd::MAV_CMD_DO_GRIPPER as u16,
            frame: mavlink::common::MAV_FRAME::MAV_FRAME_GLOBAL_RELATIVE_ALT as u8,
            altitude: 0.0,
            params: vec![
                1,
                action, // open/close
                "null",
                "null",
                "null",
                "null",
                "null",
                "null",
            ],
            ..Item::default()
        }
    }

    fn try_mode(mode: mavlink::common::MAV_MODE) -> Result<Self, ()> {
        let item = Item {
            command: MavCmd::MAV_CMD_DO_SET_MODE as u16,
            frame: mavlink::common::MAV_FRAME::MAV_FRAME_GLOBAL_RELATIVE_ALT as u8,
            altitude: 0.0,
            params: vec![
                mode as u8,
                "null", // custom mode
                "null",
                "null",
                "null",
                "null",
                "null",
                "null",
            ],
            ..Item::default()
        };

        Ok(item)
    }

    fn set_home(pose: Option<Pose>) -> Self {
        let (use_current, pose) = match pose {
            Some(pose) => (0, pose),
            None => (1, Pose {
                latitude: 0.0,
                longitude: 0.0,
                altitude: 0.0,
                heading: 0.0,
            }),
        };

        Item {
            command: MavCmd::MAV_CMD_DO_SET_HOME as u16,
            params: vec![
                use_current, // use current position
                "NaN",
                "NaN",
                "NaN",
                pose.latitude, // latitude
                pose.longitude, // longitude
                pose.altitude, // altitude m
            ],
            ..Item::default()
        }
    }

    fn payload_place(pose: Pose, expected_descent_m: u8) -> Self {
        Item {
            command: MavCmd::MAV_CMD_NAV_PAYLOAD_PLACE as u16,
            params: vec![
                expected_descent_m, // maximum distance to descend
                "null",
                "null",
                "null", // degrees
                pose.latitude, // latitude
                pose.longitude, // longitude
                pose.altitude, // altitude m
            ],
            ..Item::default()
        }
    }

    fn do_winch(
        action: mavlink::common::WINCH_ACTIONS,
        length_m: f32,
        rate_m_s: f32,
    ) -> Self {
        Item {
            command: MavCmd::MAV_CMD_DO_WINCH as u16,
            frame: mavlink::common::MAV_FRAME::MAV_FRAME_GLOBAL_RELATIVE_ALT as u8,
            altitude: 0.0,
            params: vec![
                1,
                action as u8,
                length_m,
                rate_m_s,
                "null",
                "null",
                "null"
            ],
            ..Item::default()
        }
    }

    //
    // Aim the camera at a specific point
    //
    fn set_roi(pose: Pose, gimbal_camera_id: u8) -> Self {
        Item {
            command: MavCmd::MAV_CMD_DO_SET_ROI_LOCATION as u16,
            params: vec![
                gimbal_camera_id, // gimbal device ID
                "null",
                "null",
                "null",
                pose.latitude, // latitude
                pose.longitude, // longitude
                pose.altitude, // altitude m
            ],
            ..Item::default()
        }
    }

    // fn param_set(param_id: String, param_value: f32) -> Self {
    //     Item {
    //         command: MavCmd::MAV_CMD_DO_SET_PARAMETER as u16,
    //         params: vec![
    //             0.0, // target system
    //             0.0, // target component
    //             param_id.parse().unwrap(), // parameter id
    //             param_value, // parameter value
    //             "null",
    //             "null",
    //             "null",
    //         ],
    //         ..Item::default()
    //     }
    // }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Mission {
    cruise_speed: u32,
    firmware_type: u8,
    global_plan_altitude_mode: u8,
    hover_speed: u32,
    items: Vec<Item>,
    planned_home_position: [f64; 3],
    vehicle_type: u8,
    version: u8
}

#[derive(Serialize)]
#[serde(rename = "camelCase")]
struct Circle {
    center: [f64; 2],
    radius: f64,
}

#[derive(Serialize)]
#[serde(rename = "Circle")]
struct GeoFenceCircle {
    circle: Circle,
    inclusion: bool,
    version: u8,
}

#[derive(Serialize)]
struct GeoFencePolygon {
    polygon: Vec<[f64; 2]>,
    inclusion: bool,
    version: u8,
}

#[derive(Serialize)]
#[serde(rename = "camelCase")]
struct GeoFence {
    circles: Vec<GeoFenceCircle>,
    polygons: Vec<GeoFencePolygon>,
    version: u8,
}

impl Default for GeoFence {
    fn default() -> Self {
        GeoFence {
            circles: vec![],
            polygons: vec![],
            version: GEOFENCE_VERSION,
        }
    }
}

#[derive(Serialize)]
#[serde(rename = "camelCase")]
struct RallyPoints {
    points: Vec<[f64; 3]>,
    version: u8,
}

impl Default for RallyPoints {
    fn default() -> Self {
        RallyPoints {
            points: vec![],
            version: RALLYPOINTS_VERSION,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Plan {
    file_type: String,
    geo_fence: GeoFence,
    ground_station: String,
    mission: Mission,
    rally_points: RallyPoints,
    version: u8,
}

impl super::Specification for Plan {
    /// Common start mission items
    pub try_mission_start_items(time: DateTime<Utc>, pose: Pose) -> Result<Vec<Item>, ()> {
        Ok(vec![
            // TODO(R5): turn on logging
            Item::set_home(Some(pose)),
            Item::delay(time),
            Item::try_mode(MAV_MODE::MAV_MODE_AUTO_ARMED)?,
            // TODO(R5): camera?
            // TODO(R5): PARAMs to set? See QGC parameters window
            // ODID type, session ID, etc.?
        ])
    }

    pub try_mission_end_items() -> Result<Vec<Item>, ()> {
        Ok(vec![
            Item::try_mode(MAV_MODE::MAV_MODE_AUTO_DISARMED)?,
            // TODO(R5): turn off logging
        ])
    }

    /// A hop mission is a single flight from takeoff to landing,
    ///  with no special gripper or winch activity. It is the simplest
    ///  type of mission, and is used for simple point-to-point flights.
    /// In between hops, a parcel may be loaded into the aircraft
    ///  or unloaded from the aircraft while it is disarmed.
    pub try_hop(data: super::HopData) -> Result<Self, ()> {
        let mut items: Vec<Item> = Vec::new();

        let vtol_cruise: bool = match vehicle_type {
            MAV_TYPE::MAV_TYPE_VTOL_TAILSITTER_DUOROTOR..=MAV_TYPE::MAV_TYPE_VTOL_TILTWING => true,
            MAV_TYPE::MAV_TYPE_FLAPPING_WING => true,
            _ => false,
        };

        //
        // Add common start mission items
        //
        items.extend(Self::try_mission_start_items(
            data.takeoff_time,
            data.takeoff_location
        )?);

        // TODO(R5): add MOTOR_TEST with THROTTLE_PERCENT 5%
        //  Makes it apparent to nearby individuals that a mission is about to occur
        // items.push(Item::motor_test());

        //
        // To takeoff, the aircraft should be in multirotor mode
        // 
        if vtol_cruise {
            items.push(Item::transition(MAV_VTOL_STATE::MAV_VTOL_STATE_MC)?);
        }

        //
        // TAKEOFF
        //
        let waypoint_first = data.waypoints.first().ok_or_else(|| {
            common_error!("No waypoints provided");
        })?;

        let yaw = 0.0; // TODO: Calculate this from takeoff_location and first waypoint
        let pitch = 0.0; // ^ same
        items.push(Item::takeoff(waypoint_first, yaw, pitch));

        //
        // If capable of transition to cruise, do so here
        // 
        if vtol_cruise {
            items.push(Item::try_transition(MAV_VTOL_STATE::MAV_VTOL_STATE_FW)?);
        }

        //
        // Navigate to each waypoint
        //
        let waypoint_commands = data
            .waypoints
            .iter()
            .map(|waypoint| Item::waypoint(waypoint))
            .collect::<Vec<Item>>();
        items.extend(waypoint_commands);

        //
        // If capable of transition from cruise to vertical, do so here
        // 
        if vtol_cruise {
            items.push(Item::try_transition(MAV_VTOL_STATE::MAV_VTOL_STATE_MC)?);
        }

        //
        // Set ROI to landing site
        //
        items.push(Item::set_roi(data.landing_location, data.gimbal_camera_id));

        //
        // Wait at final waypoint for landing window
        //
        items.push(Item::delay(data.landing_time));

        //
        // LAND
        //
        // TODO(R5): Calculate yaw and pitch from last waypoint and landing_location
        let waypoint_last = data.waypoints.last().ok_or_else(|| {
            common_error!("No waypoints provided");
        })?;
        items.push(Item::landing(waypoint_last, yaw, pitch));

        //
        // Common end mission items
        //
        items.extend(Self::try_mission_end_items()?);

        //
        // Important: iterate through all items and set do_jump_id
        items.iter_mut().enumerate().for_each(|(i, item)| {
            item.do_jump_id = i as u16 + 1; // starts from 1
        });

        Plan {
            file_type: "Plan".to_string(),
            geo_fence: GeoFence::default(),
            rally_points: RallyPoints::default(),
            ground_station: "AethericRealm".to_string(),
            mission: Mission {
                items,
                version: MISSION_VERSION,
                data.cruise_speed,
                data.hover_speed,
                firmware_type: data.autopilot_type as u8,
                vehicle_type: data.vehicle_type as u8,
                global_plan_altitude_mode: 1, // TODO: check this
                planned_home_position: takeoff_location
            },
            version: PLAN_VERSION,
        }
    }

    //
    // A swoop mission is a continuous mission that involves a pickup
    //  of a parcel *without landing*, and a dropoff of the parcel
    //  similarly without landing.
    pub fn try_swoop(data: super::ContinuousData) -> Result<Self, ()> {
        let mut items: Vec<Item> = Vec::new();

        let vtol_cruise: bool = match vehicle_type {
            MAV_TYPE::MAV_TYPE_VTOL_TAILSITTER_DUOROTOR..=MAV_TYPE::MAV_TYPE_VTOL_TILTWING => true,
            MAV_TYPE::MAV_TYPE_FLAPPING_WING => true,
            _ => false,
        };

        //
        // Add common start mission items
        //
        items.extend(Self::try_mission_start_items(data.takeoff_time, data.takeoff_location)?);


        // TODO(R5): add MOTOR_TEST with THROTTLE_PERCENT 5%
        //  Makes it apparent to nearby individuals that a mission is about to occur
        // items.push(Item::motor_test());

        //
        // To takeoff, the aircraft should be in multirotor mode
        // 
        if vtol_cruise {
            items.push(Item::transition(MAV_VTOL_STATE::MAV_VTOL_STATE_MC)?);
        }

        //
        // TAKEOFF
        //
        let waypoint_first = data.waypoints.first().ok_or_else(|| {
            common_error!("No waypoints provided");
        })?;

        let yaw = 0.0; // TODO: Calculate this from takeoff_location and first waypoint
        let pitch = 0.0; // ^ same
        items.push(Item::takeoff(waypoint_first, yaw, pitch));

        //
        // If capable of transition to cruise, do so here
        // 
        if vtol_cruise {
            items.push(Item::try_transition(MAV_VTOL_STATE::MAV_VTOL_STATE_FW)?);
        }

        //
        // Navigate to each waypoint on the first deadhead, if any
        //
        let waypoint_commands = data
            .waypoints_deadhead_a
            .iter()
            .map(|waypoint| Item::waypoint(waypoint))
            .collect::<Vec<Item>>();
        items.extend(waypoint_commands);

        //
        // If capable of transition from cruise to vertical, do so here
        // 
        if vtol_cruise {
            items.push(Item::try_transition(MAV_VTOL_STATE::MAV_VTOL_STATE_MC)?);
        }

        //
        // Set ROI to pickup site
        //
        items.push(Item::set_roi(data.pickup_location, data.gimbal_camera_id));

        //
        // Wait at final waypoint for pickup time
        //
        items.push(Item::delay(data.pickup_time));

        //
        // Ensure gripper is open, then navigate to pickup spot
        //
        items.push(Item::gripper(mavlink::common::GRIPPER_ACTIONS::GRIPPER_ACTION_RELEASE));
        items.push(Item::waypoint(data.pickup_location));

        //
        // Activate Gripper
        //
        items.push(Item::gripper(mavlink::common::GRIPPER_ACTIONS::GRIPPER_ACTION_GRAB));

        //
        // Ascend to first waypoint on main path
        //
        let waypoint_first = data.waypoints_main.first().ok_or_else(|| {
            common_error!("No waypoints provided");
        })?;
        items.push(Item::waypoint(waypoint_first));

        //
        // If capable of transition from vertical to cruise, do so here
        // 
        if vtol_cruise {
            items.push(Item::try_transition(MAV_VTOL_STATE::MAV_VTOL_STATE_FW)?);
        }

        //
        // Navigate to each waypoint on the main path
        //
        let waypoint_commands = data
            .waypoints_main
            .iter()
            .map(|waypoint| Item::waypoint(waypoint))
            .collect::<Vec<Item>>();

        //
        // If capable of transition from cruise to vertical, do so here
        // 
        if vtol_cruise {
            items.push(Item::try_transition(MAV_VTOL_STATE::MAV_VTOL_STATE_MC)?);
        }

        //
        // Set ROI to dropoff site
        //
        items.push(Item::set_roi(data.dropoff_location, data.gimbal_camera_id));

        //
        // Wait at final waypoint for dropoff time
        //
        items.push(Item::delay(data.dropoff_time));

        //
        // If gentle dropoff, place payload on the ground at dropoff location
        //
        if data.gentle_dropoff {
            items.push(Item::payload_place(data.dropoff_location));
        } else {
            //
            // Go to dropoff spot
            //
            items.push(Item::waypoint(data.dropoff_location));

            //
            // Deactivate Gripper (even if from a height)
            //
            items.push(Item::gripper(mavlink::common::GRIPPER_ACTIONS::GRIPPER_ACTION_RELEASE));
        }

        //
        // Ascend to first waypoint on the second deadhead
        //
        let waypoint_first = data.waypoints_deadhead_b.first().ok_or_else(|| {
            common_error!("No waypoints provided");
        })?;
        items.push(Item::waypoint(waypoint_first));

        //
        // If capable of transition from vertical to cruise, do so here
        //
        if vtol_cruise {
            items.push(Item::try_transition(MAV_VTOL_STATE::MAV_VTOL_STATE_FW)?);
        }

        //
        // Navigate to each waypoint on the second deadhead
        //
        let waypoint_commands = data
            .waypoints_deadhead_b
            .iter()
            .map(|waypoint| Item::waypoint(waypoint))
            .collect::<Vec<Item>>();

        //
        // If capable of transition from cruise to vertical, do so here
        //
        if vtol_cruise {
            items.push(Item::try_transition(MAV_VTOL_STATE::MAV_VTOL_STATE_MC)?);
        }

        //
        // Set ROI to landing site
        //
        items.push(Item::set_roi(data.landing_location, data.gimbal_camera_id));


        //
        // Wait at the final waypoint for landing window
        // 
        items.push(Item::delay(data.landing_time));

        //
        // LAND
        //
        items.push(Item::landing(data.landing_location, yaw, pitch));

        //
        // Common end mission items
        //
        items.extend(Self::try_mission_end_items()?);

        //
        // Important: iterate through all items and set do_jump_id
        items.iter_mut().enumerate().for_each(|(i, item)| {
            item.do_jump_id = i as u16 + 1; // starts from 1
        });

        Plan {
            file_type: "Plan".to_string(),
            geo_fence: GeoFence::default(),
            rally_points: RallyPoints::default(),
            ground_station: "AethericRealm".to_string(),
            mission: Mission {
                items,
                version: MISSION_VERSION,
                data.cruise_speed,
                data.hover_speed,
                firmware_type: data.autopilot_type as u8,
                vehicle_type: data.vehicle_type as u8,
                global_plan_altitude_mode: 1, // TODO: check this
                planned_home_position: takeoff_location
            },
            version: PLAN_VERSION,
        }
    }

    /// A winch mission is a continuous mission that involves a winch
    /// TODO(R5): This is largely dependent on the type of winch release
    ///  mechanism. Do not use yet.
    pub fn try_winch(data: super::ContinuousData) -> Result<Self, ()> {
        unimplemented!();
        return Err(());

        let mut items: Vec<Item> = Vec::new();

        let vtol_cruise: bool = match vehicle_type {
            MAV_TYPE::MAV_TYPE_VTOL_TAILSITTER_DUOROTOR..=MAV_TYPE::MAV_TYPE_VTOL_TILTWING => true,
            MAV_TYPE::MAV_TYPE_FLAPPING_WING => true,
            _ => false,
        };

        // Reel in the winch if not already
        // The winch should be fully retracted before takeoff
        // TODO(R5): Not sure if this or RETRACT is the correct command
        // If the winch is caught on something, this could be dangerous.
        //  Would drag the aircraft across the ground while it is arming
        //  and trying to take off.
        items.push(Item::do_winch(mavlink::common::WINCH_ACTIONS::WINCH_LOAD_LINE, 0.0, 0.0)); // other arguments ignored

        //
        // Add common start mission items
        //
        items.extend(Self::try_mission_start_items(data.takeoff_time, data.takeoff_location)?);

        // TODO(R5): add MOTOR_TEST with THROTTLE_PERCENT 5%
        //  Makes it apparent to nearby individuals that a mission is about to occur
        // items.push(Item::motor_test());

        //
        // To takeoff, the aircraft should be in multirotor mode
        // 
        if vtol_cruise {
            items.push(Item::transition(MAV_VTOL_STATE::MAV_VTOL_STATE_MC)?);
        }

        // TODO(R5): add MOTOR_TEST with THROTTLE_PERCENT 5%
        //  Makes it apparent to nearby individuals that a mission is about to occur
        // items.push(Item::motor_test());

        //
        // To takeoff, the aircraft should be in multirotor mode
        // 
        if vtol_cruise {
            items.push(Item::transition(MAV_VTOL_STATE::MAV_VTOL_STATE_MC)?);
        }

        //
        // TAKEOFF
        //
        let waypoint_first = data.waypoints.first().ok_or_else(|| {
            common_error!("No waypoints provided");
        })?;
        let yaw = 0.0; // TODO: Calculate this from takeoff_location and first waypoint
        let pitch = 0.0; // ^ same
        items.push(Item::takeoff(waypoint_first, yaw, pitch));

        //
        // If capable of transition to cruise, do so here
        // 
        if vtol_cruise {
            items.push(Item::try_transition(MAV_VTOL_STATE::MAV_VTOL_STATE_FW)?);
        }

        //
        // Navigate to each waypoint on the first deadhead, if any
        //
        let waypoint_commands = data
            .waypoints_deadhead_a
            .iter()
            .map(|waypoint| Item::waypoint(waypoint))
            .collect::<Vec<Item>>();
        items.extend(waypoint_commands);

        //
        // If capable of transition from cruise to vertical, do so here
        // 
        if vtol_cruise {
            items.push(Item::try_transition(MAV_VTOL_STATE::MAV_VTOL_STATE_MC)?);
        }

        //
        // Wait at final waypoint for pickup time
        //
        items.push(Item::delay(data.pickup_time));

        //
        // Go to pickup spot
        //
        items.push(Item::waypoint(data.pickup_location));

        //
        // Activate Gripper
        //
        items.push(Item::gripper(mavlink::common::GRIPPER_ACTIONS::GRIPPER_ACTION_GRAB));

        //
        // Ascend to first waypoint on main path
        //
        let waypoint_first = data.waypoints_main.first().ok_or_else(|| {
            common_error!("No waypoints provided");
        })?;
        items.push(Item::waypoint(waypoint_first));

        //
        // If capable of transition from vertical to cruise, do so here
        // 
        if vtol_cruise {
            items.push(Item::try_transition(MAV_VTOL_STATE::MAV_VTOL_STATE_FW)?);
        }

        //
        // Navigate to each waypoint on the main path
        //
        let waypoint_commands = data
            .waypoints_main
            .iter()
            .map(|waypoint| Item::waypoint(waypoint))
            .collect::<Vec<Item>>();

        //
        // If capable of transition from cruise to vertical, do so here
        // 
        if vtol_cruise {
            items.push(Item::try_transition(MAV_VTOL_STATE::MAV_VTOL_STATE_MC)?);
        }

        //
        // Wait at final waypoint for dropoff time
        //
        items.push(Item::delay(data.dropoff_time));

        //
        // Go to dropoff spot
        //
        items.push(Item::waypoint(Pose {
            altitude: data.dropoff_location.altitude + data.winch_height,
            ..data.dropoff_location.clone()
        }));

        //
        // Lower Winch
        //
        items.push(Item::do_winch(mavlink::common::WINCH_ACTIONS::WINCH_DELIVER, 0.0, 0.0)); // other arguments ignored

        //
        // Ascend to first waypoint on the second deadhead
        //
        let waypoint_first = data.waypoints_deadhead_b.first().ok_or_else(|| {
            common_error!("No waypoints provided");
        })?;
        items.push(Item::waypoint(waypoint_first));

        //
        // If capable of transition from vertical to cruise, do so here
        //
        if vtol_cruise {
            items.push(Item::try_transition(MAV_VTOL_STATE::MAV_VTOL_STATE_FW)?);
        }

        //
        // Navigate to each waypoint on the second deadhead
        //
        let waypoint_commands = data
            .waypoints_deadhead_b
            .iter()
            .map(|waypoint| Item::waypoint(waypoint))
            .collect::<Vec<Item>>();

        //
        // If capable of transition from cruise to vertical, do so here
        //
        if vtol_cruise {
            items.push(Item::try_transition(MAV_VTOL_STATE::MAV_VTOL_STATE_MC)?);
        }

        //
        // Wait at the final waypoint for landing window
        // 
        items.push(Item::delay(data.landing_time));

        //
        // LAND
        //
        items.push(Item::landing(data.landing_location, yaw, pitch));

        //
        // DISARM
        //
        items.extend(Self::try_mission_end_items()?);

        //
        // Important: iterate through all items and set do_jump_id
        items.iter_mut().enumerate().for_each(|(i, item)| {
            item.do_jump_id = i as u16 + 1; // starts from 1
        });

        Plan {
            file_type: "Plan".to_string(),
            geo_fence: GeoFence::default(),
            rally_points: RallyPoints::default(),
            ground_station: "AethericRealm".to_string(),
            mission: Mission {
                items,
                version: MISSION_VERSION,
                data.cruise_speed,
                data.hover_speed,
                firmware_type: data.autopilot_type as u8,
                vehicle_type: data.vehicle_type as u8,
                global_plan_altitude_mode: 1, // TODO: check this
                planned_home_position: takeoff_location
            },
            version: PLAN_VERSION,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_whole_to_json() {
        let mut rng = rand::thread_rng();

        let circle_center_longitude = rng.gen_range(-180.0..180.0);
        let circle_center_latitude = rng.gen_range(-90.0..90.0);
        let circle_center_radius = rng.gen_range(0.0..1000.0);

        let polygon_vertices = (0..4).map(|_| {
            [rng.gen_range(-90.0..90.0), rng.gen_range(-180.0..180.0)]
        }).collect::<Vec<[f64; 2]>>();

        let polygon_vertices_str = polygon_vertices.iter().map(|v| {
            format!("[{}, {}]", v[0], v[1])
        }).collect::<Vec<String>>().join(", ");

        // Item
        let item_params = (0..6).map(|_| rng.gen_range(0.0..100.0)).collect::<Vec<f64>>();
        let item_params_str = item_params.iter().map(|p| p.to_string()).collect::<Vec<String>>().join(", ");
        let command = MavCmd::MAV_CMD_NAV_WAYPOINT.to_u16().unwrap();
        let frame = rng.gen_range(0..100);
        let do_jump_id = rng.gen_range(0..100);
        let altitude = rng.gen_range(0.0..100.0);
        let altitude_mode = rng.gen_range(0..100);
        let amsl_alt_above_terrain = rng.gen_range(0.0..100.0);
        let auto_continue = rng.gen_bool(0.5);

        let cruise_speed = rng.gen_range(0..100);
        let firmware_type = rng.gen_range(0..100);
        let hover_speed = rng.gen_range(0..100);

        let planned_home_position = [rng.gen_range(-90.0..90.0), rng.gen_range(-180.0..180.0), rng.gen_range(0.0..1000.0)];
        let planned_home_position_str = format!("{}, {}, {}", planned_home_position[0], planned_home_position[1], planned_home_position[2]);

        let expected = format!(r#"
        {{
            "fileType": "Plan",
            "geoFence": {{
                "circles": [
                    {{
                        "circle": {{
                            "center": [{circle_center_latitude}, {circle_center_longitude}],
                            "radius": {circle_center_radius}
                        }},
                        "inclusion": true,
                        "version": 1
                    }}
                ],
                "polygons": [
                    {{
                        "polygon": [{polygon_vertices_str}],
                        "inclusion": true,
                        "version": 1
                    }}
                ],
                "version": {GEOFENCE_VERSION}
            }},
            "groundStation": "QGroundControl",
            "mission": {{
                "cruiseSpeed": {cruise_speed},
                "firmwareType": {firmware_type},
                "globalPlanAltitudeMode": 1,
                "hoverSpeed": {hover_speed},
                "items": [
                    {{
                        "type": "SimpleItem",
                        "autoContinue": {auto_continue},
                        "command": {command},
                        "doJumpId": {do_jump_id},
                        "frame": {frame},
                        "params": [{item_params_str}],
                        "Altitude": {altitude},
                        "AltitudeMode": {altitude_mode},
                        "AMSLAltAboveTerrain": {amsl_alt_above_terrain}
                    }}
                ],
                "plannedHomePosition": [{planned_home_position_str}],
                "vehicleType": 2,
                "version": {MISSION_VERSION}
            }},
            "rallyPoints": {{
                "points": [],
                "version": {RALLYPOINTS_VERSION}
            }},
            "version": 1
        }}"#, command=command as u16)
        .replace("\n", "")
        .replace(" ", "");

        let plan = Plan {
            file_type: "Plan".to_string(),
            ground_station: "QGroundControl".to_string(),
            version: 1,
            mission: Mission {
                cruise_speed,
                firmware_type,
                global_plan_altitude_mode: 1,
                hover_speed,
                items: vec![Item {
                    type_: "SimpleItem".to_string(),
                    auto_continue,
                    command,
                    do_jump_id,
                    frame,
                    params: item_params,
                    altitude,
                    altitude_mode,
                    amsl_alt_above_terrain
                }],
                planned_home_position,
                vehicle_type: 2,
                version: MISSION_VERSION,
            },
            rally_points: RallyPoints::default(),
            geo_fence: GeoFence {
                circles: vec![
                    GeoFenceCircle {
                        circle: Circle {
                            center: [circle_center_latitude, circle_center_longitude],
                            radius: circle_center_radius
                        },
                        inclusion: true,
                        version: 1
                    }
                ],
                polygons: vec![
                    GeoFencePolygon {
                        inclusion: true,
                        polygon: polygon_vertices,
                        version: 1
                    }
                ],
                version: GEOFENCE_VERSION
            }
        };

        let json = serde_json::to_string(&plan).unwrap();
        assert_eq!(json, expected);
    }
}
