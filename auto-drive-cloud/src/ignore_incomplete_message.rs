use log::{Level, Log, Metadata, Record};

// Required because of long lived SSE. Issue described well here in this open ticket: https://github.com/hyperium/hyper/issues/3759

pub struct IgnoreIncompleteMessage<L>(pub L);

impl<L: Log> Log for IgnoreIncompleteMessage<L> {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.0.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if record.metadata().level() == Level::Error
            && record.metadata().target() == "warp::server::run"
            && record.args().to_string().contains("IncompleteMessage")
        {
            return;
        }
        self.0.log(record);
    }

    fn flush(&self) {
        self.0.flush();
    }
}
