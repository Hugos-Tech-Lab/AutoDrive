use std::sync::mpsc::{self, Receiver, SyncSender};

use esp_idf_svc::log::EspIdfLogger;
use log::{LevelFilter, Log, Metadata, Record};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogMessage {
    pub level: u8,
    pub target: String,
    pub message: String,
}

struct Logger {
    inner: EspIdfLogger<()>,
    sender: SyncSender<LogMessage>,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let message = LogMessage {
            level: record.level() as u8,
            target: record.target().to_owned(),
            message: record.args().to_string(),
        };

        // Don't block the application if the I2C consumer is slow.
        let _ = self.sender.try_send(message);

        // Also keep the normal ESP-IDF serial logging.
        self.inner.log(record);
    }

    fn flush(&self) {
        self.inner.flush();
    }
}

pub fn init_logging() -> Receiver<LogMessage> {
    let (sender, receiver) = mpsc::sync_channel(100);

    let logger = Box::leak(Box::new(Logger {
        inner: EspIdfLogger::new(()),
        sender,
    }));

    log::set_logger(logger)
        .expect("failed to initialize logger");

    log::set_max_level(LevelFilter::Debug);

    receiver
}
