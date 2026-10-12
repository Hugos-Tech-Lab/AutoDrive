
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub enum TheBeginnerCarIncomingMessages {
    SetMotorASpeed { speed: i8 },
    SetMotorBSpeed { speed: i8 },
    InformPositionFromCamera { x: i32, y: i32 },
    RunAutoScript { data: Vec<u8> },
    GetBatteryReading,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TheBeginnerCarOutgoingMessages {
    Logs { logs: Vec<()> },
    SystemState {},
    MotorASpeedUpdated { speed: i8 },
    MotorBSpeedUpdated { speed: i8 },
    WasmInstalled,
    WasmRunning,
    BatteryReading { voltage: f32 },
}

/// Serializes a message with a 4-byte big-endian length prefix.
pub fn to_bytes<T: Serialize>(
    message: &T,
) -> Result<Vec<u8>, Box<dyn Error>> {
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