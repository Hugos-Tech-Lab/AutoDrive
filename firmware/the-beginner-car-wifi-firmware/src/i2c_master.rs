use std::time::{Duration, Instant};

use esp_idf_svc::hal::{
    delay::{BLOCK, FreeRtos}, gpio::{AnyIOPin, Gpio12, Gpio21, Input, Pin, PinDriver, Pull}, i2c::{I2c, I2cConfig, I2cDriver}, units::Hertz,
};
use log::{error, info};
use serde::{Deserialize, Serialize};

/// Must match the slave address.
const SLAVE_ADDR: u8 = 0x22;

/// Must match the slave's TX/RX buffers to ensure we read enough bytes to get the 0x00 terminator.
const MAX_MESSAGE_SIZE: usize = 256;
const MASTER_BUFFER_SIZE: usize = 128;
const TX_FRAME_SIZE: usize = MAX_MESSAGE_SIZE + 16;

// -------------------------------------------------------------------------
// SHARED TYPES (Ideally, move these to a separate shared crate)
// -------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct LogMessage {
    pub level: u8,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    LightOn,
    LightOff,
    SetMotorSpeed(i32),
    SetMotorSpeedFor { motor: u8, speed: i32 },
    ReadRegister { address: u8 },
    WriteRegister { address: u8, value: u8 },
    Logs,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Ok,
    Error,
    RegisterValue { address: u8, value: u8 },
    Status { light_on: bool, motor_speed: i32 },
    Logs { logs: Vec<LogMessage> },
}

// -------------------------------------------------------------------------
// MASTER IMPLEMENTATION
// -------------------------------------------------------------------------

/// Initializes the I2C peripheral as a Master.
pub fn i2c_master_init<'d>(
    i2c: impl I2c + 'd,
    sda: AnyIOPin<'d>,
    scl: AnyIOPin<'d>,
    baudrate: u32,
) -> anyhow::Result<I2cDriver<'d>> {
    let config = I2cConfig::new().baudrate(Hertz(baudrate)).scl_enable_pullup(true).sda_enable_pullup(true);
        dbg!("set baud rate");

    let driver = I2cDriver::new(i2c, sda, scl, &config)?;
        dbg!("new drvier");

    Ok(driver)
}

/// Sends a COBS-encoded Postcard request to the slave and waits for the response.
pub fn send_request(i2c: &mut I2cDriver<'_>, request: &Request, ready_pin: &mut PinDriver<'_, esp_idf_svc::hal::gpio::Input>) -> anyhow::Result<Response> {

    let mut tx_frame = [0u8; TX_FRAME_SIZE];

    // 1. Encode the request using Postcard + COBS
    let encoded = postcard::to_slice_cobs(request, &mut tx_frame)
        .map_err(|e| anyhow::anyhow!("Failed to encode request: {e:?}"))?;

    i2c.write(SLAVE_ADDR, encoded, BLOCK)?;

    let timeout = Duration::from_millis(150); // Maximum time we will wait
    let start_time = Instant::now();

    while ready_pin.is_low() {
        if start_time.elapsed() > timeout {
            return Err(anyhow::anyhow!("I2C Timeout: Slave did not pull DRDY high"));
        }
        // Yield to the RTOS for 1 tick so we don't spinlock the CPU
        FreeRtos::delay_ms(1);
    }

    // 4. Read the response chunk
    let mut rx_buf = [0u8; MASTER_BUFFER_SIZE];
    i2c.read(SLAVE_ADDR, &mut rx_buf, BLOCK)?;

    // 5. Find the COBS terminator (0x00)
    // The slave's logic drops the 0x00 before decoding, so we must do the same here.
    let cobs_end = rx_buf
        .iter()
        .position(|&byte| byte == 0)
        .ok_or_else(|| anyhow::anyhow!("COBS terminator 0x00 not found in read buffer"))?;

    let frame = &mut rx_buf[..cobs_end];

    // 6. Decode the response
    let response: Response = postcard::from_bytes_cobs(frame)
        .map_err(|e| anyhow::anyhow!("Failed to decode response: {e:?}"))?;

    Ok(response)
}

// /// Example master control loop demonstrating communication with the slave.
// pub fn run_master_loop(i2c: &mut I2cDriver<'_>) -> anyhow::Result<()> {
//     // A sequence of commands to test the slave logic
//     let sequence = [
//         Request::LightOn,
//         Request::SetMotorSpeed(1500),
//         Request::WriteRegister {
//             address: 0x10,
//             value: 0xAA,
//         },
//         Request::ReadRegister { address: 0x10 },
//         Request::LightOff,
//     ];

//     loop {
//         for request in &sequence {
//             info!("MASTER: Sending request: {request:?}");

//             match send_request(i2c, request) {
//                 Ok(response) => info!("MASTER: Received response: {response:?}"),
//                 Err(e) => error!("MASTER: I2C transaction failed: {}", e),
//             }

//             // Wait a bit before sending the next command
//             FreeRtos::delay_ms(1000);
//         }
//     }
// }
