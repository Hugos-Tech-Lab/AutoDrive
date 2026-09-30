use std::sync::mpsc::{self, Receiver, SyncSender};

use esp_idf_svc::{
    hal::{
        delay::BLOCK,
        gpio::AnyIOPin,
        i2c::{I2c, I2cSlaveConfig, I2cSlaveDriver},
        peripherals::Peripherals,
    },
    log::EspIdfLogger,
};

use log::{Level, LevelFilter, Log, Metadata, Record};
use serde::{Deserialize, Serialize};

use crate::logger::LogMessage;

const LOG_CHANNEL_SIZE: usize = 32;
const LOG_BUFFER_SIZE: usize = 256;

const SLAVE_ADDR: u8 = 0x22;
const SLAVE_BUFFER_SIZE: usize = 128;

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

    let driver = I2cSlaveDriver::new(
        i2c,
        sda,
        scl,
        slave_addr,
        &config,
    )?;

    Ok(driver)
}

/// Blocks forever, sending queued log messages to the I2C master.
///
/// Each message is encoded as:
///
///     LogMessage
///         ↓
///     Postcard
///         ↓
///     COBS
///         ↓
///     I2C
pub fn send_logs_over_i2c(
    peripherals: Peripherals,
    log_receiver: Receiver<LogMessage>,
) -> anyhow::Result<()> {
    let mut i2c_slave = i2c_slave_init(
        peripherals.i2c0,
        peripherals.pins.gpio18.into(),
        peripherals.pins.gpio19.into(),
        SLAVE_BUFFER_SIZE,
        SLAVE_ADDR,
    )?;

    let mut buffer = [0u8; LOG_BUFFER_SIZE];

    loop {
        let log = match log_receiver.recv() {
            Ok(log) => log,
            Err(_) => {
                println!("I2C LOG: log channel closed");
                break;
            }
        };

        let encoded = match postcard::to_slice_cobs(
            &log,
            &mut buffer,
        ) {
            Ok(encoded) => encoded,

            Err(error) => {
                println!(
                    "I2C LOG: failed to serialize log: {error:?}"
                );
                continue;
            }
        };

        if let Err(error) = i2c_slave.write(encoded, BLOCK) {
            println!(
                "I2C LOG: failed to send log over I2C: {error:?}"
            );
        }
    }

    Ok(())
}
