use std::sync::{Arc, Mutex, OnceLock};

use crate::device_control::DeviceControl;

static HARDWARE: OnceLock<Arc<Mutex<Hardware>>> = OnceLock::new();

pub struct Hardware {
  //
}

impl Hardware {
  pub fn control_wheel() {
    //
  }

  pub fn read_wheel() {
    //
  }
}
