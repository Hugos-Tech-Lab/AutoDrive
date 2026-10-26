use auto_drive_common_types::LogMessage;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutoScript {
    pub run_uuid: String,
    pub preprocessed_file_hash: String,
    pub run_number: u32,
    pub device_id: String,
    pub script: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TheBeginnerCarIncomingMessages {
    SetMotorASpeed { speed: i8 },
    SetMotorBSpeed { speed: i8 },
    InformPositionFromCamera { x: i32, y: i32 },
    CancelAutoScript { run_uuid: String },
    RunAutoScript { script: AutoScript },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TheBeginnerCarOutgoingMessages {
    Log {
        log: LogMessage,
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
        voltage: f32,
    },
    InfraredLedReadings {
        back_center: u16,
        front_left: u16,
        front_center: u16,
        front_right: u16,
    },
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
