use embassy_time::with_timeout;
use esp_idf_svc::hal::{
    gpio::{AnyIOPin, Input, Output, Pin, PinDriver},
    spi::{SpiAnyPins, SpiDeviceDriver, SpiDriver, SpiDriverConfig, config},
    task::block_on,
    units::Hertz,
};
use serde::{Serialize, de::DeserializeOwned};

use crate::spi_packet::{PACKET_SIZE, SpiPacket, SpiPackets};

pub struct SpiMaster<'d> {
    device_driver: SpiDeviceDriver<'d, SpiDriver<'d>>,
    slave_ready: PinDriver<'d, Input>,
    master_has_transaction: PinDriver<'d, Output>,
    used_pins: Box<[u8]>,
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
        mut f_master_has_transaction_pin: PinDriver<'d, Output>,
        baudrate: u32,
    ) -> anyhow::Result<Self> {
        let mut used_pins = [
            a_sclk.pin(),
            b_mosi.pin(),
            c_miso.pin(),
            d_slave_ready_pin.pin(),
            e_cs.pin(),
            f_master_has_transaction_pin.pin(),
        ];
        used_pins.sort();
        let driver_config = SpiDriverConfig::new();
        let driver = SpiDriver::new(spi, a_sclk, b_mosi, Some(c_miso), &driver_config)?;
        let device_config = config::Config::new().baudrate(Hertz(baudrate));
        let device_driver = SpiDeviceDriver::new(driver, Some(e_cs), &device_config)?;

        f_master_has_transaction_pin.set_low().unwrap();

        Ok(Self {
            device_driver,
            slave_ready: d_slave_ready_pin,
            master_has_transaction: f_master_has_transaction_pin,
            used_pins: Box::new(used_pins),
        })
    }

    pub fn wait_until_ready_pin(slave_ready: &mut PinDriver<'_, Input>, is_high: bool) -> anyhow::Result<()> {
        let pin_number = slave_ready.pin();

        let fut = async {
            if is_high {
                slave_ready.wait_for_high().await
            } else {
                slave_ready.wait_for_low().await
            }
        };

        match block_on(with_timeout(embassy_time::Duration::from_secs(1), fut)) {
            Ok(result) => {
                // let used_pins = format!("{:?}", self.used_pins);
                result.map_err(|err| anyhow::anyhow!("Failed to wait until ready pin: '{err:?}'. Check physical pin connection number"))
            }
            Err(_) => {
                // let used_pins = format!("{:?}", self.used_pins);

                Err(anyhow::anyhow!(
                    "Timeout waiting for READY pin ({pin_number}) to be ({}). Check physical pin connection number",
                    if is_high { "HIGH" } else { "LOW" }
                ))
            }
        }?;

        Ok(())
    }

    pub fn wait_until_slave_doesnt_want_data(slave_ready: &mut PinDriver<'_, Input>) -> anyhow::Result<()> {
        Self::wait_until_ready_pin(slave_ready, false)
    }

    pub fn wait_until_slave_wants_data(slave_ready: &mut PinDriver<'_, Input>) -> anyhow::Result<()> {
        Self::wait_until_ready_pin(slave_ready, true)
    }

    pub fn send_request<TReq, TRes>(&mut self, request: &TReq) -> anyhow::Result<TRes>
    where
        TRes: DeserializeOwned,
        TReq: Serialize,
    {
        let encoded = postcard::to_allocvec(request)
            .map_err(|e| anyhow::anyhow!("Failed to encode request: {e:?}"))?;
        let packets = SpiPackets::from_payload(&encoded)
            .map_err(|err| anyhow::anyhow!("Could not construct payload: {:?}", err))?;
        for packet in packets.iter() {
            let payload = packet.to_bytes();
            Self::wait_until_slave_wants_data(&mut self.slave_ready)?;
            let _master_transaction = MasterTransaction::new(&mut self.master_has_transaction);
            self.device_driver.write(&payload)?;
            Self::wait_until_slave_doesnt_want_data(&mut self.slave_ready)?;
        }

        let mut response = Vec::<SpiPacket>::new();
        loop {
            Self::wait_until_slave_wants_data(&mut self.slave_ready)?;
            let _master_transaction = MasterTransaction::new(&mut self.master_has_transaction);
            let mut packet = [0u8; PACKET_SIZE];
            self.device_driver.read(&mut packet)?;

            let packet = SpiPacket::from_bytes(&packet)
                .map_err(|err| anyhow::anyhow!("Could not reconstruct payload: {:?}", err))?;
            let packet = response.push_mut(packet);
            Self::wait_until_slave_doesnt_want_data(&mut self.slave_ready)?;

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

struct MasterTransaction<'a, 'd> {
    master_has_transaction: &'a mut PinDriver<'d, Output>,
}

impl<'a, 'd> MasterTransaction<'a, 'd> {
    pub fn new(
        master_has_transaction: &'a mut PinDriver<'d, Output>,
    ) -> Self {
        master_has_transaction.set_high().unwrap();

        Self {
            master_has_transaction,
        }
    }
}

impl Drop for MasterTransaction<'_, '_> {
    fn drop(&mut self) {
        self.master_has_transaction.set_low().unwrap();
    }
}
