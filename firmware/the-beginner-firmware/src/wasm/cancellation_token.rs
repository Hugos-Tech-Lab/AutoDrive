use std::sync::{atomic::{AtomicBool, Ordering}};

static WASM_CANCELLED: AtomicBool = AtomicBool::new(false);

pub fn cancel() {
    WASM_CANCELLED.store(true, Ordering::Relaxed);
}

pub fn is_cancelled() -> bool {
    WASM_CANCELLED.load(Ordering::Relaxed)
}

pub fn reset() {
    WASM_CANCELLED.store(false, Ordering::Relaxed);
}

