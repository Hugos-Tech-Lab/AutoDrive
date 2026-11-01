use std::collections::HashMap;

use auto_drive_common_types::LogMessage;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CalibrationReadings {
    left_motor_raw_to_rpm: HashMap<i8, u16>,
    right_motor_raw_to_rpm: HashMap<i8, u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestToHardware {
    LightOn,
    LightOff,
    SetMotorLeftSpeed(i8),
    SetMotorRightSpeed(i8),
    DrainEvents,
    NotifyPositionFromCamera {
        x: i32,
        y: i32,
        rotation_radians: f32,
    },
    NotifyBatteryReadings {
        voltage: f32,
    },
    RunAutoScript {
        data: Vec<u8>,
    },
    StopAutoScript,
    RunCalibration,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Log {
        message: LogMessage,
    },
    InfraredLedReadings {
        back_center: u16,
        front_left: u16,
        front_center: u16,
        front_right: u16,
    },
    AutoScriptStopped,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseFromHardware {
    Ok,
    Status { light_on: bool, motor_speed: i32 },
    Events { events: Vec<Event> },
    AutoScriptStarted { id: i32 },
    CalibrationReadings {},
}
