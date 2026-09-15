use std::{
    collections::VecDeque,
    sync::{Mutex, OnceLock},
};

use esp_idf_svc::log::EspIdfLogger;
use log::{LevelFilter, Log, Metadata, Record};

const MAX_LOGS: usize = 500;

pub static LOG_BUFFER: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

struct BufferedLogger {
    inner: EspIdfLogger<()>,
}

static LOGGER: BufferedLogger = BufferedLogger {
    inner: EspIdfLogger::new(()),
};

impl Log for BufferedLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let line = format!(
            "[{}] {}: {}",
            record.level(),
            record.target(),
            record.args()
        );

        if let Some(buffer) = LOG_BUFFER.get() {
            if let Ok(mut buffer) = buffer.lock() {
                if buffer.len() >= MAX_LOGS {
                    buffer.pop_front();
                }

                buffer.push_back(line);
            }
        }

        // Also send the log to the normal ESP-IDF serial logger.
        self.inner.log(record);
    }

    fn flush(&self) {
        self.inner.flush();
    }
}

pub fn init_logging() {
    LOG_BUFFER
        .set(Mutex::new(VecDeque::with_capacity(MAX_LOGS)))
        .ok();

    log::set_logger(&LOGGER).expect("failed to initialize logger");
    log::set_max_level(LevelFilter::Debug);
}
