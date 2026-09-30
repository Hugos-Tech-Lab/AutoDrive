use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

#[derive(Clone)]
pub struct CancellationToken {
    raw: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            raw: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.raw.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.raw.load(Ordering::Relaxed)
    }

    pub fn reset(&self) {
        self.raw.store(false, Ordering::Relaxed);
    }
}
