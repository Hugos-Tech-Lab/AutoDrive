use std::sync::atomic::{AtomicU32, Ordering};

use the_beginner_car_spi_protocol::AutoScriptState;

pub struct AtomicAutoScriptState(AtomicU32);

impl AtomicAutoScriptState {
    pub const fn new(state: AutoScriptState) -> Self {
        Self(AtomicU32::new(state as u32))
    }

    pub fn store(&self, state: AutoScriptState) {
        self.0.store(state as u32, Ordering::Release);
    }

    pub fn load(&self) -> AutoScriptState {
        unsafe { std::mem::transmute::<u32, AutoScriptState>(self.0.load(Ordering::Acquire)) }
    }
}
