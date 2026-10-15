use std::sync::mpsc::{self, Receiver, SyncSender};

use esp_idf_svc::log::EspIdfLogger;
use log::{LevelFilter, Log, Metadata, Record};
use the_beginner_car_spi_protocol::{LogLevel, LogMessage};

use std::cell::RefCell;

thread_local! {
    // Clippy false positive I guess? Open issue: https://github.com/rust-lang/rust-clippy/issues/13422
    #[allow(clippy::missing_const_for_thread_local)]
    static TRACE_ID: RefCell<Option<String>> =
        const { RefCell::new(None) };
}

pub fn set_trace_id(id: impl Into<String>) {
    TRACE_ID.with(|trace_id| {
        *trace_id.borrow_mut() = Some(id.into());
    });
}

pub fn clear_trace_id() {
    TRACE_ID.with(|trace_id| {
        *trace_id.borrow_mut() = None;
    });
}

fn current_trace_id() -> Option<String> {
    TRACE_ID.with(|trace_id| trace_id.borrow().clone())
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

        let level = match record.level() {
            log::Level::Error => LogLevel::Error,
            log::Level::Warn => LogLevel::Warn,
            log::Level::Info => LogLevel::Info,
            log::Level::Debug => LogLevel::Debug,
            log::Level::Trace => LogLevel::Trace,
        };

        let message = LogMessage {
            level,
            target: record.target().to_owned(),
            message: record.args().to_string(),
            trace_id: current_trace_id(),
        };

        // Don't block the application if the SPI consumer is slow.
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
        sender: sender.clone(),
    }));

    log::set_logger(logger)
        .expect("failed to initialize logger");

    log::set_max_level(LevelFilter::Debug);

    receiver
}
