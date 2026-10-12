use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogMessage {
    pub level: u8,
    pub target: String,
    pub message: String,
    pub trace_id: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestToHardware {
    LightOn,
    LightOff,
    SetMotorASpeed(i8),
    SetMotorBSpeed(i8),
    DrainEvents,
    NotifyPositionFromCamera {
        x: i32,
        y: i32,
    },
    NotifyBatteryReadings {
        motor_battery: f32,
        mcu_battery: f32,
    },
    RunAutoScript {
        data: Vec<u8>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Log { message: LogMessage },
    IRSensorA { value: u16 },
    AutoScriptStopped
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseFromHardware {
    Ok,
    Status { light_on: bool, motor_speed: i32 },
    Events { logs: Vec<LogMessage> },
    AutoScriptStarted { id: i32 },
}
