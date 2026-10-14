use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LogSystemOrigin {
    WasmMcu,
    WifiMcu
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogMessage {
    pub system_origin: LogSystemOrigin,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub trace_id: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TheBeginnerCarIncomingMessages {
    SetMotorASpeed { speed: i8 },
    SetMotorBSpeed { speed: i8 },
    InformPositionFromCamera { x: i32, y: i32 },
    StopAutoScript,
    RunAutoScript { data: Vec<u8> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TheBeginnerCarOutgoingMessages {
    Log {
        log: LogMessage
    },
    SystemState {},
    MotorASpeedUpdated {
        speed: i8,
    },
    MotorBSpeedUpdated {
        speed: i8,
    },
    WasmInstalled,
    AutoScriptRunning,
    AutoScriptStopped,
    BatteryReading {
        motor_battery_voltage: f32,
        mcu_battery_voltage: f32,
    },
    InfraredLedReadings {
        utmost_front_utmost_left: u16,
        utmost_front_utmost_right: u16
    }
}

/// Serializes a message with a 4-byte big-endian length prefix.
pub fn to_bytes<T: Serialize>(message: &T) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut result = vec![0u8; 4];

    result = postcard::to_extend(message, result)?;

    let payload_len = u32::try_from(result.len() - 4)?;
    result[..4].copy_from_slice(&payload_len.to_be_bytes());

    Ok(result)
}

/// Convenience trait for message types.
pub trait MessageCodec: Serialize + DeserializeOwned + Sized {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        to_bytes(self)
    }
}

impl MessageCodec for TheBeginnerCarIncomingMessages {}
impl MessageCodec for TheBeginnerCarOutgoingMessages {}
