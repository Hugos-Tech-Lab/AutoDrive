use std::{sync::mpsc::Receiver, time::Duration};

use edge_http::io::server::Server;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        delay::BLOCK,
        gpio::{AnyIOPin, PinDriver},
        i2c::{I2c, I2cSlaveConfig, I2cSlaveDriver},
        ledc::{LedcDriver, LedcTimerDriver, config::TimerConfig},
        peripherals::Peripherals,
        spi::{Dma, SpiBusDriver, SpiConfig, SpiDriver, SpiDriverConfig},
        units::Hertz,
    },
};

use log::info;
use serde::{Deserialize, Serialize};

use crate::logger::LogMessage;

const SLAVE_ADDR: u8 = 0x22;

/// Maximum amount of data that can be transferred in one I2C message.
const MAX_MESSAGE_SIZE: usize = 256;

/// I2C hardware RX/TX buffers.
///
/// These are deliberately smaller than MAX_MESSAGE_SIZE to demonstrate
/// that messages can arrive over multiple I2C reads.
const SLAVE_BUFFER_SIZE: usize = 128;

/// Temporary buffer used to accumulate a complete COBS frame.
const RX_FRAME_SIZE: usize = MAX_MESSAGE_SIZE + 16;

/// Temporary buffer used to serialize a response.
const TX_FRAME_SIZE: usize = MAX_MESSAGE_SIZE + 16;

/// Messages sent from the I2C master to this device.
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Turn the light on.
    LightOn,

    /// Turn the light off.
    LightOff,

    /// Set motor speed.
    SetMotorSpeed(i32),

    /// Set motor speed with an explicit motor ID.
    SetMotorSpeedFor {
        motor: u8,
        speed: i32,
    },

    /// Read a register.
    ReadRegister {
        address: u8,
    },

    /// Write a register.
    WriteRegister {
        address: u8,
        value: u8,
    },

    Logs,
}

/// Messages sent from this device back to the I2C master.
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    /// Command was successfully executed.
    Ok,

    /// Command failed.
    Error,

    /// Response containing a register value.
    RegisterValue {
        address: u8,
        value: u8,
    },

    /// Generic status information.
    Status {
        light_on: bool,
        motor_speed: i32,
    },

    Logs {
        logs: Vec<LogMessage>,
    },
}

fn i2c_slave_init<'d>(
    i2c: impl I2c + 'd,
    sda: AnyIOPin<'d>,
    scl: AnyIOPin<'d>,
    buflen: usize,
    slave_addr: u8,
) -> anyhow::Result<I2cSlaveDriver<'d>> {
    let config = I2cSlaveConfig::new()
        .rx_buffer_length(buflen)
        .tx_buffer_length(buflen);

    let driver = I2cSlaveDriver::new(i2c, sda, scl, slave_addr, &config)?;

    Ok(driver)
}

pub fn receive_loop<'d>(
    i2c: impl I2c + 'd,
    sda: AnyIOPin<'d>,
    scl: AnyIOPin<'d>,
    log_message_receiver: Receiver<LogMessage>,
) -> anyhow::Result<()> {
    let mut i2c_slave = i2c_slave_init(i2c, sda, scl, SLAVE_BUFFER_SIZE, SLAVE_ADDR)?;

    // Example device state.
    let mut registers = [0u8; 256];
    let mut light_on = false;
    let mut motor_speed: i32 = 0;

    // A single I2C read may only give us part of a message.
    //
    // For example:
    //
    //     I2C read #1: [01, 03, 80]
    //     I2C read #2: [FE, 01, 00]
    //
    // The 0x00 terminates the COBS frame.
    let mut rx_frame = [0u8; RX_FRAME_SIZE];
    let mut rx_len = 0usize;

    // Buffer used for Postcard + COBS encoded responses.
    let mut tx_frame = [0u8; TX_FRAME_SIZE];

    // Temporary buffer for each I2C chunk.
    let mut chunk = [0u8; SLAVE_BUFFER_SIZE];

    loop {
        info!("loop waiting");

        match i2c_slave.read(&mut chunk, BLOCK) {
            Ok(n) => {
                info!("GOT BYTES");

                // We received bytes from the master.
                //
                // IMPORTANT:
                // `n` is the number of bytes actually received.
                for &byte in &chunk[..n] {
                    if byte == 0 {
                        if rx_len == 0 {
                            continue;
                        }

                        let frame = &mut rx_frame[..rx_len];

                        match postcard::from_bytes_cobs::<Request>(frame) {
                            Ok(request) => {
                                println!("SLAVE: request: {request:?}");

                                let response = handle_request(
                                    request,
                                    &mut registers,
                                    &mut light_on,
                                    &mut motor_speed,
                                    &log_message_receiver,
                                );

                                println!("SLAVE: response: {response:?}");

                                match postcard::to_slice_cobs(&response, &mut tx_frame) {
                                    Ok(encoded) => {
                                        if let Err(e) = i2c_slave.write(encoded, BLOCK) {
                                            println!("SLAVE: failed to write response: {e:?}");
                                        }
                                    }
                                    Err(e) => {
                                        println!("SLAVE: failed to encode response: {e:?}");
                                    }
                                }
                            }

                            Err(e) => {
                                println!("SLAVE: failed to decode Postcard frame: {e:?}");
                            }
                        }

                        rx_len = 0;
                    } else {
                        if rx_len >= rx_frame.len() {
                            println!("SLAVE: RX frame too large, discarding frame");
                            rx_len = 0;
                            continue;
                        }

                        rx_frame[rx_len] = byte;
                        rx_len += 1;
                    }
                }
            }

            Err(e) => {
                // With the ESP I2C slave driver, this is also where the
                // master-side transaction can indicate that it wants to
                // read from us, depending on the driver/transaction state.
                //
                // The important thing is that any complete COBS frames
                // received above have already been decoded and responded to.
                println!("SLAVE: I2C read ended: {e:?}");
            }
        }
    }
}

fn handle_request(
    request: Request,
    registers: &mut [u8; 256],
    light_on: &mut bool,
    motor_speed: &mut i32,
    log_message_receiver: &Receiver<LogMessage>,
) -> Response {
    match request {
        Request::LightOn => {
            *light_on = true;

            println!("SLAVE: light ON");

            Response::Ok
        }

        Request::LightOff => {
            *light_on = false;

            println!("SLAVE: light OFF");

            Response::Ok
        }

        Request::SetMotorSpeed(speed) => {
            *motor_speed = speed;

            println!("SLAVE: motor speed = {speed}");

            Response::Ok
        }

        Request::SetMotorSpeedFor { motor, speed } => {
            println!("SLAVE: motor {motor} speed = {speed}");

            // Do whatever actual motor control is required here.

            Response::Ok
        }

        Request::ReadRegister { address } => {
            let value = registers[address as usize];

            println!("SLAVE: read register {address:#04x} -> {value:#04x}");

            Response::RegisterValue { address, value }
        }

        Request::WriteRegister { address, value } => {
            registers[address as usize] = value;

            println!("SLAVE: write register {address:#04x} <- {value:#04x}");

            Response::Ok
        }
        Request::Logs => {
            info!("hi");

            let logs = log_message_receiver.try_iter().collect();
            Response::Logs { logs }
        }
    }
}
