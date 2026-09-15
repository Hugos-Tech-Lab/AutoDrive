use std::{ffi::c_void, thread, time::Duration};

use log::info;

#[unsafe(no_mangle)]
pub extern "C" fn delay(_exec_env: *mut c_void, milliseconds: u64) {
    thread::sleep(Duration::from_millis(milliseconds));
}

#[unsafe(no_mangle)]
pub extern "C" fn print(_exec_env: *mut c_void, text: u32) {
    info!("received: {:?}", text);
}
