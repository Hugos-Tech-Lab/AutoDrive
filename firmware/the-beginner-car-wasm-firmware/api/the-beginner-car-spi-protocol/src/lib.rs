use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogMessage {
    pub level: LogLevel,
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
    StopAutoScript
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Log { message: LogMessage },
    InfraredLedReadings {
        utmost_front_utmost_left: u16,
        utmost_front_utmost_right: u16
    },
    AutoScriptStopped
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseFromHardware {
    Ok,
    Status { light_on: bool, motor_speed: i32 },
    Events { events: Vec<Event> },
    AutoScriptStarted { id: i32 }
}
