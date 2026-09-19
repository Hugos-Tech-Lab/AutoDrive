use std::{thread, time::Duration};

use log::info;
use smart_leds_trait::RGB8;
use wamr_rust_sdk::sys::{WASMExecEnv, wasm_runtime_get_module_inst, wasm_runtime_terminate};

use crate::{hardware::on_board_led::OnBoardLed, auto_script::cancellation_token};

pub fn terminate(exec_env: *mut WASMExecEnv) {
    if exec_env.is_null() {
        return;
    }
    let instance = unsafe { wasm_runtime_get_module_inst(exec_env) };
    unsafe { wasm_runtime_terminate(instance) };
}

macro_rules! check_cancellation {
    ($exec_env:expr) => {
        if cancellation_token::is_cancelled() {
            terminate($exec_env);
            return;
        }
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn delay(exec_env: *mut WASMExecEnv, milliseconds: u8) {
    check_cancellation!(exec_env);
    thread::sleep(Duration::from_millis(milliseconds as u64));
}

#[unsafe(no_mangle)]
pub extern "C" fn print(exec_env: *mut WASMExecEnv, text: u32) {
    check_cancellation!(exec_env);
    info!("received: {:?}", text);
}

#[unsafe(no_mangle)]
pub extern "C" fn set_onboard_led_color(exec_env: *mut WASMExecEnv, r: u8, g: u8, b: u8) {
    check_cancellation!(exec_env);
    OnBoardLed::set_color(RGB8 { r, g, b })
}
