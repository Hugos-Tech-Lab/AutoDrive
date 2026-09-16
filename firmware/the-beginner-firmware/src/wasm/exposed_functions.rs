use std::{ffi::c_void, thread, time::Duration};

use log::info;
use smart_leds_trait::RGB8;

use crate::hardware::on_board_led::OnBoardLed;

#[unsafe(no_mangle)]
pub extern "C" fn delay(_exec_env: *mut c_void, milliseconds: u64) {
    thread::sleep(Duration::from_millis(milliseconds));
}

#[unsafe(no_mangle)]
pub extern "C" fn print(_exec_env: *mut c_void, text: u32) {
    info!("received: {:?}", text);
}

#[unsafe(no_mangle)]
pub extern "C" fn set_onboard_led_color(_exec_env: *mut c_void, r: u8, g: u8, b: u8) {
    OnBoardLed::set_color(RGB8 { r, g, b })
}
