use std::time::{Duration, Instant};

use esp_idf_svc::hal::{
    delay::FreeRtos, gpio::{AnyIOPin, Input, Output, PinDriver}, spi::{SpiAnyPins, SpiDeviceDriver, SpiDriver, SpiDriverConfig, config}, units::Hertz,
};
use serde::{Serialize, de::DeserializeOwned};

use crate::spi_packet::{PACKET_SIZE, SpiPacket, SpiPackets};

pub struct SpiMaster<'d> {
    device_driver: SpiDeviceDriver<'d, SpiDriver<'d>>,
    slave_ready: PinDriver<'d, Input>,
    master_has_transaction: PinDriver<'d, Output>,
}

impl<'d> SpiMaster<'d> {
    /// SPI pins:
    ///
    /// SCLK -> clock
    /// MOSI -> master out / slave in
    /// MISO -> master in / slave out
    /// CS   -> chip select
    pub fn new<SPI: SpiAnyPins + 'd>(
        spi: SPI,
        a_sclk: AnyIOPin<'d>,
        b_mosi: AnyIOPin<'d>,
        c_miso: AnyIOPin<'d>,
        d_slave_ready_pin: PinDriver<'d, Input>,
        e_cs: AnyIOPin<'d>,
        f_master_ready_pin: PinDriver<'d, Output>,
        baudrate: u32,
    ) -> anyhow::Result<Self> {
        let driver_config = SpiDriverConfig::new();
        let driver = SpiDriver::new(spi, a_sclk, b_mosi, Some(c_miso), &driver_config)?;
        let device_config = config::Config::new().baudrate(Hertz(baudrate));
        let device_driver = SpiDeviceDriver::new(driver, Some(e_cs), &device_config)?;
        Ok(Self {
            device_driver,
            slave_ready: d_slave_ready_pin,
            master_has_transaction: f_master_ready_pin
        })
    }

    pub fn wait_until_ready_pin(&self, is_high: bool) -> anyhow::Result<()> {
        let start = Instant::now();
        let timeout = Duration::from_millis(15000);

        while self.slave_ready.is_high() != is_high {
            if start.elapsed() > timeout {
                return Err(anyhow::anyhow!("SPI timeout waiting for pin {}", if is_high {
                    "high"
                } else {
                    "low"
                }));
            }

            FreeRtos::delay_ms(1);
        }

        Ok(())
    }

    pub fn wait_until_slave_doesnt_want_data(&self) -> anyhow::Result<()> {
        self.wait_until_ready_pin(false)
    }

    pub fn wait_until_slave_wants_data(&self) -> anyhow::Result<()> {
        self.wait_until_ready_pin(true)
    }

    pub fn send_request<TReq, TRes>(&mut self, request: &TReq) -> anyhow::Result<TRes>
    where
        TRes: DeserializeOwned,
        TReq: Serialize,
    {
        let encoded = postcard::to_allocvec(request)
            .map_err(|e| anyhow::anyhow!("Failed to encode request: {e:?}"))?;
        let packets = SpiPackets::from_payload(&encoded).unwrap();
        for packet in packets.iter() {
            let payload = packet.to_bytes();
            self.wait_until_slave_wants_data()?;
            self.master_has_transaction.set_high().unwrap();
            self.device_driver.write(&payload).unwrap();
            self.wait_until_slave_doesnt_want_data()?;
            self.master_has_transaction.set_low().unwrap();
        }

        let mut response = Vec::<SpiPacket>::new();
        loop {
            self.wait_until_slave_wants_data()?;
            self.master_has_transaction.set_high().unwrap();
            let mut packet = [0u8; PACKET_SIZE];
            self.device_driver.read(&mut packet)?;

            let packet = SpiPacket::from_bytes(&packet).unwrap();
            let packet = response.push_mut(packet);
            self.wait_until_slave_doesnt_want_data()?;
            self.master_has_transaction.set_low().unwrap();

            if packet.is_last() {
                break;
            }
        }
        let response = SpiPackets::from_vec(response);
        let response: TRes = postcard::from_bytes(&response.payload())
            .map_err(|e| anyhow::anyhow!("Failed to decode response: {e:?}"))?;

        Ok(response)
    }
}
