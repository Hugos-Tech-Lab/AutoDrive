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
    SetMotorASpeed(i8),
    SetMotorBSpeed(i8),
    Logs,
    NotifyPositionFromCamera { x: i32, y: i32 },
    NotifyBatteryReadings { motor_battery: f32, mcu_battery: f32 },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseFromHardware {
    Ok,
    Status { light_on: bool, motor_speed: i32 },
    Logs { logs: Vec<LogMessage> },
}
