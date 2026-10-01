use std::{sync::mpsc::Receiver, time::Duration};

use edge_http::io::server::Server;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop, hal::{
        delay::{FreeRtos, TickType}, gpio::{AnyIOPin, Gpio21, PinDriver, Pull}, i2c::{I2c, I2cSlaveConfig, I2cSlaveDriver}, ledc::{LedcDriver, LedcTimerDriver, config::TimerConfig}, peripherals::Peripherals, spi::{Dma, SpiBusDriver, SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig}, units::Hertz,
    },
};

use log::info;
use serde::{Deserialize, Serialize};

use crate::logger::LogMessage;

const SLAVE_ADDR: u8 = 0x22;

/// Maximum amount of data that can be transferred in one I2C message.
const MAX_MESSAGE_SIZE: usize = 256;

/// I2C hardware RX/TX buffers.
const SLAVE_BUFFER_SIZE: usize = 128;

/// Temporary buffer used to accumulate a complete COBS frame.
const RX_FRAME_SIZE: usize = MAX_MESSAGE_SIZE + 16;

/// Temporary buffer used to serialize a response.
const TX_FRAME_SIZE: usize = MAX_MESSAGE_SIZE + 16;

/// Messages sent from the I2C master to this device.
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

/// Messages sent from this device back to the I2C master.
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Ok,
    Error,
    RegisterValue { address: u8, value: u8 },
    Status { light_on: bool, motor_speed: i32 },
    Logs { logs: Vec<LogMessage> },
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
    gpio_pin21: Gpio21,
) -> anyhow::Result<()> {
    let mut i2c_slave = i2c_slave_init(i2c, sda, scl, SLAVE_BUFFER_SIZE, SLAVE_ADDR)?;

    let mut ready_pin: PinDriver<'_, esp_idf_svc::hal::gpio::Output> =
        PinDriver::output(gpio_pin21)?;

    // let mut device_1 = SpiDeviceDriver::new(&driver, Some(cs_1), &config_1)?;

    ready_pin.set_low().unwrap();
    let mut registers = [0u8; 256];
    let mut light_on = false;
    let mut motor_speed: i32 = 0;

    let mut rx_frame = [0u8; RX_FRAME_SIZE];
    let mut rx_len = 0usize;

    let mut tx_frame = [0u8; TX_FRAME_SIZE];
    let mut chunk = [0u8; SLAVE_BUFFER_SIZE];

    // 10ms timeout prevents the bus from blocking indefinitely

    loop {
        match i2c_slave.read(&mut chunk, TickType::from(Duration::from_millis(10)).into()) {
            Ok(n) if n > 0 => {
                // n is the number of bytes actually received.
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

                                // Push the response to the TX buffer so the master can read it
                                match postcard::to_slice_cobs(&response, &mut tx_frame) {
                                    Ok(encoded) => {
                                        if let Err(e) = i2c_slave.write(encoded, 1000) {
                                            println!("SLAVE: failed to write response: {e:?}");
                                        }
                                        ready_pin.set_high().unwrap();

                                        FreeRtos::delay_ms(10);

                                        let _ = ready_pin.set_low();
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

                        // IMPORTANT: continue instead of break to process remaining bytes in the chunk!
                        continue;
                    }

                    if rx_len >= rx_frame.len() {
                        println!("SLAVE: RX frame too large, discarding frame");
                        rx_len = 0;
                        // IMPORTANT: continue instead of break!
                        continue;
                    }

                    rx_frame[rx_len] = byte;
                    rx_len += 1;
                }
            }
            Ok(_) => {
                // 0 bytes read. The bus is quiet, just loop around.
            }
            Err(_e) => {
                // Timeouts throw an error in ESP-IDF. We intentionally ignore them
                // here so the console isn't spammed every 10ms.
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
            let logs = log_message_receiver.try_iter().collect();
            Response::Logs { logs }
        }
    }
}
