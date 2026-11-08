use std::collections::HashMap;

use auto_drive_common_types::LogMessage;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CalibrationReadings {
    left_motor_raw_to_rpm: HashMap<i8, u16>,
    right_motor_raw_to_rpm: HashMap<i8, u16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,
    pub timestamp_unix_epoch_ms: u64,
}

impl Position {
    pub fn add(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.rotation_radians += other.rotation_radians;
        self.timestamp_unix_epoch_ms = other.timestamp_unix_epoch_ms;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestToHardware {
    LightOn,
    LightOff,
    SetMotorLeftSpeed(i8),
    SetMotorRightSpeed(i8),
    DrainEvents,
    GetPosition,
    NotifyPositionFromCamera { position: Position },
    NotifyBatteryReadings { voltage: f32 },
    RunAutoScript { data: Vec<u8> },
    StopAutoScript,
    RunCalibration,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Log {
        message: LogMessage,
    },
    InfraredLedReadings {
        front_far_left: u16,
        front_left: u16,
        front_center: u16,
        front_right: u16,
        front_far_right: u16,
        back_left: u16,
        back_center: u16,
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
    Position { position: Option<Position> },
}
