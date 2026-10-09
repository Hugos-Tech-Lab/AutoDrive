use std::{
    collections::VecDeque,
    sync::{Mutex, OnceLock},
};

use esp_idf_svc::log::EspIdfLogger;
use log::{LevelFilter, Log, Metadata, Record};

const MAX_LOGS: usize = 100;

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

        let file = record.file().unwrap_or("unknown");
        let line = record
            .line()
            .map(|n| n.to_string())
            .unwrap_or_else(|| "?".to_string());

        let entry = format!(
            "[{}] [{}:{}] {}: {}",
            record.level(),
            file,
            line,
            record.target(),
            record.args()
        );

        // Print the enriched message to serial output.
        println!("{}", entry);

        // Store the same message for the HTTP logs endpoint.
        if let Some(buffer) = LOG_BUFFER.get() {
            if let Ok(mut buffer) = buffer.lock() {
                if buffer.len() >= MAX_LOGS {
                    buffer.pop_front();
                }

                buffer.push_back(entry);
            }
        }
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
