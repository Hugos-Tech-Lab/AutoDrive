use esp_idf_svc::{
    hal::gpio::{Output, PinDriver},
    sys::*,
};
use std::{ffi::c_void, marker::PhantomData, ptr};

use crate::{
    spi_master::{Request, Response},
    spi_packet::{PACKET_SIZE, SpiPacket, SpiPackets},
};

pub struct SpiSlave {
    host: spi_host_device_t,
    _not_send_sync: PhantomData<*mut ()>,
}

impl SpiSlave {
    pub fn new(
        host: spi_host_device_t,
        bus_config: spi_bus_config_t,
        slave_config: spi_slave_interface_config_t,
        dma_chan: spi_dma_chan_t,
    ) -> Result<Self, EspError> {
        let err = unsafe { spi_slave_initialize(host, &bus_config, &slave_config, dma_chan) };
        if let Some(err) = EspError::from(err) {
            return Err(err);
        }

        Ok(Self {
            host,
            _not_send_sync: PhantomData,
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

    fn read(
        &self,
        ready_pin: &mut PinDriver<'_, Output>,
    ) -> anyhow::Result<Box<[u8; PACKET_SIZE]>> {
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

        self.queue_trans(&transaction).unwrap();

        ready_pin.set_high().unwrap();

        let _ = self.trans_result(&mut transaction).unwrap();

        ready_pin.set_low().unwrap();

        Ok(Box::new(rx))
    }

    fn write(
        &self,
        ready_pin: &mut PinDriver<'_, Output>,
        data: Box<[u8; PACKET_SIZE]>,
    ) -> anyhow::Result<()> {
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

        self.queue_trans(&transaction).unwrap();

        ready_pin.set_high();

        let _ = self.trans_result(&mut transaction).unwrap();

        ready_pin.set_low();

        Ok(())
    }

    pub fn handle_requests(&self, ready_pin: &mut PinDriver<'_, Output>) {
        let mut request = Vec::<SpiPacket>::new();
        loop {
            let packet = self.read(ready_pin).unwrap();
            if packet.iter().all(|&b| b == 0) {
                println!("RX contains only zeros");
                continue;
            }
            let packet = SpiPacket::from_bytes(packet.as_ref()).unwrap();
            let packet = request.push_mut(packet);
            if packet.is_last() {
                break;
            }
        }
        let request = SpiPackets::from_vec(request);
        let request: Request = postcard::from_bytes(&request.payload()).unwrap();
        // TODO: proper handling
        match request {
            Request::LightOn => println!("LightOn"),
            Request::LightOff => println!("LightOff"),
            Request::SetMotorSpeed(_) => println!("SetMotorSpeed"),
            Request::SetMotorSpeedFor { motor, speed } => println!("SetMotorSpeedFor"),
            Request::ReadRegister { address } => println!("ReadRegister"),
            Request::WriteRegister { address, value } => println!("WriteRegister"),
            Request::Logs => println!("logs"),
        }

        let response = postcard::to_allocvec(&Response::Ok).unwrap();
        let packets = SpiPackets::from_payload(&response).unwrap();
        for packet in packets.iter() {
            self.write(ready_pin, Box::new(packet.to_bytes()));
        }
    }
}

impl Drop for SpiSlave {
    fn drop(&mut self) {
        unsafe {
            spi_slave_free(self.host);
        }
    }
}
