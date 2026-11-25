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
pub struct IrSensorArray {
    // front
    pub front_far_left: u32,
    pub front_left: u32,
    pub front_center: u32,
    pub front_right: u32,
    pub front_far_right: u32,
    // back
    pub back_center: u32,
    pub back_left: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(u32)]
pub enum AutoScriptState {
    Idle,
    ReceivedByWasmMcu,
    CreatingWasmModule,
    InstantiatingWasmModule,
    PreparingToRunWasmMain,
    FindingWasmMain,
    RunningWasmMain,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestToHardware {
    LightOn,
    LightOff,
    FlushLogs,
    GetPosition,
    GetAutoScriptState,
    NotifyPositionFromCamera { position: Position },
    NotifyBatteryReadings { voltage: f32 },
    NotifyIrSensorReadings { readings: IrSensorArray },
    RunAutoScript { data: Vec<u8> },
    StopAutoScript,
    Uptime,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseFromHardware {
    Ok,
    Status { light_on: bool, motor_speed: i32 },
    Logs { logs: Vec<LogMessage> },
    AutoScriptState { state: AutoScriptState },
    AutoScriptStarted { id: i32 },
    CalibrationReadings {},
    Position { position: Option<Position> },
    Uptime { ms: u32 },
}
