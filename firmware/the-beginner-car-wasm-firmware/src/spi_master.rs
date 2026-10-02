use std::time::{Duration, Instant};

use esp_idf_svc::hal::{
    delay::FreeRtos,
    gpio::{AnyIOPin, Input, PinDriver},
    spi::{SPI2, SpiDeviceDriver, SpiDriver, SpiDriverConfig, config},
    units::Hertz,
};
use serde::{Deserialize, Serialize};

use crate::spi_packet::{PACKET_SIZE, SpiPacket, SpiPackets};

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

pub struct SpiMaster<'d> {
    device_driver: SpiDeviceDriver<'d, SpiDriver<'d>>,
}

impl<'d> SpiMaster<'d> {
    /// SPI pins:
    ///
    /// SCLK -> clock
    /// MOSI -> master out / slave in
    /// MISO -> master in / slave out
    /// CS   -> chip select
    pub fn new(
        spi: SPI2<'d>,
        sclk: AnyIOPin<'d>,
        mosi: AnyIOPin<'d>,
        miso: AnyIOPin<'d>,
        cs: AnyIOPin<'d>,
        baudrate: u32,
    ) -> anyhow::Result<Self> {
        let driver_config = SpiDriverConfig::new();
        let driver = SpiDriver::new(spi, sclk, mosi, Some(miso), &driver_config)?;
        let device_config = config::Config::new().baudrate(Hertz(baudrate));
        let device_driver = SpiDeviceDriver::new(driver, Some(cs), &device_config)?;
        Ok(Self { device_driver })
    }

    pub fn send_request(
        &mut self,
        request: &Request,
        ready_pin: &mut PinDriver<'_, Input>,
    ) -> anyhow::Result<Response> {
        let encoded = postcard::to_allocvec(request)
            .map_err(|e| anyhow::anyhow!("Failed to encode request: {e:?}"))?;

        let packets = SpiPackets::from_payload(&encoded).unwrap();
        for packet in packets.iter() {
            wait_until_pin_is_high(ready_pin)?;
            self.device_driver.write(packet.payload());
            wait_until_pin_is_low(ready_pin)?;
        }

        let mut response = Vec::<SpiPacket>::new();
        loop {
            wait_until_pin_is_high(ready_pin)?;
            let mut packet = [0u8; PACKET_SIZE];
            self.device_driver.read(&mut packet)?;

            let packet = SpiPacket::from_bytes(&packet).unwrap();
            let packet = response.push_mut(packet);
            wait_until_pin_is_low(ready_pin)?;

            if packet.is_last() {
                break;
            }
        }
        let response = SpiPackets::from_vec(response);
        let response: Response = postcard::from_bytes(&response.payload())
            .map_err(|e| anyhow::anyhow!("Failed to decode response: {e:?}"))?;

        Ok(response)
    }
}

pub fn wait_until_pin_is_low(ready_pin: &mut PinDriver<'_, Input>) -> anyhow::Result<()> {
    let timeout = Duration::from_millis(150);
    let start_time = Instant::now();
    while ready_pin.is_low() {
        if start_time.elapsed() > timeout {
            return Err(anyhow::anyhow!("SPI timeout: slave did not pull DRDY high"));
        }

        // Yield to FreeRTOS instead of busy-spinning.
        FreeRtos::delay_ms(1);
    }
    Ok(())
}

pub fn wait_until_pin_is_high(ready_pin: &mut PinDriver<'_, Input>) -> anyhow::Result<()> {
    let timeout = Duration::from_millis(150);
    let start_time = Instant::now();
    while ready_pin.is_high() {
        if start_time.elapsed() > timeout {
            return Err(anyhow::anyhow!("SPI timeout: slave did not pull DRDY high"));
        }

        // Yield to FreeRTOS instead of busy-spinning.
        FreeRtos::delay_ms(1);
    }
    Ok(())
}
