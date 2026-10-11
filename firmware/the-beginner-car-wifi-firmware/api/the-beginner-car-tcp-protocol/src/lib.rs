use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum TheBeginnerCarIncomingMessages {
    SetMotorASpeed { speed: i8 },
    SetMotorBSpeed { speed: i8 },
    InformPositionFromCamera { x: i32, y: i32 },
    RunAutoScript { data: Vec<u8> },
    GetBatteryReading
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TheBeginnerCarOutgoingMessages {
    Logs { logs: Vec<()> },
    SystemState {},
    MotorASpeedUpdated { speed: i8 },
    MotorBSpeedUpdated { speed: i8 },
    WasmInstalled,
    WasmRunning,
    BatteryReading { voltage: f32 }
}
