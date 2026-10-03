use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogMessage {
    pub level: u8,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestToHardware {
    LightOn,
    LightOff,
    SetMotorSpeed(i32),
    SetMotorSpeedFor { motor: u8, speed: i32 },
    Logs,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseFromHardware {
    Ok,
    Status { light_on: bool, motor_speed: i32 },
    Logs { logs: Vec<LogMessage> },
}
