use embassy_time::with_timeout;
use esp_idf_svc::{
    hal::{
        gpio::{Input, InputPin, Output, OutputPin, PinDriver, Pull},
        spi::SPI2,
    },
    sys::*,
};
use futures::executor::block_on;
use serde::{Serialize, de::DeserializeOwned};
use std::{ffi::c_void, marker::PhantomData, ptr, thread, time::Duration};

use crate::spi_packet::{PACKET_SIZE, SpiPacket, SpiPackets};

pub struct SpiSlave<'d> {
    host: spi_host_device_t,
    _not_send_sync: PhantomData<*mut ()>,
    slave_wants_data: PinDriver<'d, Output>,
    master_ready: PinDriver<'d, Input>,
}

impl<'d> SpiSlave<'d> {
    pub fn new(
        _spi_2: SPI2,
        a_sclk: impl InputPin + 'd,
        b_sdi: impl InputPin + 'd,
        c_sdo: impl OutputPin + 'd,
        d_slave_ready_pin: impl OutputPin + 'd,
        e_cs: impl InputPin + 'd,
        f_master_ready_pin: impl InputPin + 'd,
    ) -> Result<Self, EspError> {
        let slave_wants_data = PinDriver::output(d_slave_ready_pin)?;
        let master_ready = PinDriver::input(f_master_ready_pin, Pull::Down)?;

        let bus_config = spi_bus_config_t {
            __bindgen_anon_1: spi_bus_config_t__bindgen_ty_1 {
                data0_io_num: b_sdi.pin() as _, // SDI (slave) - MOSI (master)
            },
            __bindgen_anon_2: spi_bus_config_t__bindgen_ty_2 {
                data1_io_num: c_sdo.pin() as _,
            },
            sclk_io_num: a_sclk.pin() as _,
            max_transfer_sz: 256,
            ..Default::default()
        };

        let slave_config = spi_slave_interface_config_t {
            spics_io_num: e_cs.pin() as _,
            queue_size: 1,
            mode: 0,
            flags: 0,
            post_setup_cb: None,
            post_trans_cb: None,
            ..Default::default()
        };

        let host = spi_host_device_t_SPI2_HOST;
        let err = unsafe {
            spi_slave_initialize(
                host,
                &bus_config,
                &slave_config,
                spi_common_dma_t_SPI_DMA_CH_AUTO,
            )
        };
        if let Some(err) = EspError::from(err) {
            return Err(err);
        }

        Ok(Self {
            host,
            _not_send_sync: PhantomData,
            slave_wants_data,
            master_ready,
        })
    }

    fn queue_trans(&self, transaction: &spi_slave_transaction_t) -> Result<(), EspError> {
        let err: esp_err_t = unsafe { spi_slave_queue_trans(self.host, transaction, u32::MAX) };
        if let Some(err) = EspError::from(err) {
            return Err(err);
        }

        Ok(())
    }

    fn trans_result(&self, transaction: &mut spi_slave_transaction_t) -> Result<usize, EspError> {
        let mut transaction_ptr: *mut spi_slave_transaction_t = transaction;

        let err: esp_err_t =
            unsafe { spi_slave_get_trans_result(self.host, &mut transaction_ptr, u32::MAX) };
        if let Some(err) = EspError::from(err) {
            return Err(err);
        }

        Ok((transaction.trans_len as usize + 7) / 8)
    }

    pub fn wait_until_master_ready_pin(&mut self, is_high: bool) -> anyhow::Result<()> {
        let fut = async {
            if is_high {
                self.master_ready.wait_for_high().await
            } else {
                self.master_ready.wait_for_low().await
            }
        };

        match block_on(with_timeout(embassy_time::Duration::from_secs(1), fut)) {
            Ok(result) => {
                result.map_err(|err| anyhow::anyhow!("Failed to wait until ready pin: '{err:?}'"))
            }
            Err(_) => Err(anyhow::anyhow!(
                "Timeout waiting for READY pin {}",
                if is_high { "HIGH" } else { "LOW" }
            )),
        }?;

        Ok(())
    }

    pub fn wait_until_master_has_no_transaction(&mut self) -> anyhow::Result<()> {
        self.wait_until_master_ready_pin(false)
    }

    fn read(&mut self) -> anyhow::Result<Box<[u8; PACKET_SIZE]>> {
        let mut rx = [0u8; PACKET_SIZE];
        let tx = [0u8; PACKET_SIZE];

        assert!(
            tx.len() >= rx.len(),
            "TX buffer must be at least as large as RX buffer"
        );

        let mut transaction = spi_slave_transaction_t {
            length: (tx.len() * 8) as usize,
            trans_len: 0,
            rx_buffer: rx.as_mut_ptr() as *mut c_void,
            tx_buffer: tx.as_ptr() as *const c_void,
            user: ptr::null_mut(),
            flags: 0,
        };

        self.wait_until_master_has_no_transaction()?; // maybe move this above?
        self.queue_trans(&transaction).unwrap();

        self.slave_wants_data.set_high().unwrap();

        let _ = self.trans_result(&mut transaction).unwrap();

        self.slave_wants_data.set_low().unwrap();

        Ok(Box::new(rx))
    }

    fn write(&mut self, data: Box<[u8; PACKET_SIZE]>) -> anyhow::Result<()> {
        let mut rx = [0u8; PACKET_SIZE];

        assert!(
            data.len() >= rx.len(),
            "TX buffer must be at least as large as RX buffer"
        );

        let mut transaction = spi_slave_transaction_t {
            length: (data.len() * 8) as usize,
            trans_len: 0,
            rx_buffer: rx.as_mut_ptr() as *mut c_void,
            tx_buffer: data.as_ptr() as *const c_void,
            user: ptr::null_mut(),
            flags: 0,
        };

        self.wait_until_master_has_no_transaction().unwrap();
        self.queue_trans(&transaction).unwrap();

        self.slave_wants_data.set_high().unwrap();

        let _ = self.trans_result(&mut transaction).unwrap();

        self.slave_wants_data.set_low().unwrap();

        Ok(())
    }

    pub fn listen<TReq, TRes, F>(&mut self, handler: F)
    where
        TReq: DeserializeOwned,
        TRes: Serialize,
        F: Fn(TReq) -> TRes,
    {
        'requests: loop {
            let mut request = Vec::<SpiPacket>::new();
            loop {
                let packet = match self.read() {
                    Ok(packet) => packet,
                    Err(e) => {
                        log::error!("Cannot read packet: {}", e);
                        continue 'requests;
                    }
                };
                let packet = match SpiPacket::from_bytes(packet.as_ref()) {
                    Ok(packet) => packet,
                    Err(e) => {
                        log::error!("Cannot decode packet: '{:?}'. Is the SPI bridge OKAY?", e);
                        thread::sleep(Duration::from_millis(1));
                        continue 'requests;
                    }
                };

                let packet = request.push_mut(packet);
                if packet.is_last() {
                    break;
                }
            }
            let request = SpiPackets::from_vec(request);
            let request: TReq = match postcard::from_bytes(&request.payload()) {
                Ok(packet) => packet,
                Err(e) => {
                    log::error!("Cannot get request from bytes: {:?}", e);
                    continue 'requests;
                }
            };

            let response = handler(request);

            let response = match postcard::to_allocvec(&response) {
                Ok(bytes) => bytes,
                Err(e) => {
                    log::error!("Cannot serialize response: {:?}", e);
                    continue 'requests;
                }
            };

            let packets = SpiPackets::from_payload(&response).unwrap();
            for packet in packets.iter() {
                self.write(Box::new(packet.to_bytes())).unwrap();
            }
        }
    }
}

impl<'d> Drop for SpiSlave<'d> {
    fn drop(&mut self) {
        unsafe {
            spi_slave_free(self.host);
        }
    }
}
