use std::{sync::Mutex, thread, time::Duration};

use flume::Sender;
use log::info;
use smart_leds_trait::RGB8;
use wamr_rust_sdk::sys::{WASMExecEnv, wasm_runtime_get_module_inst, wasm_runtime_terminate};

use crate::{auto_script::{AutoScriptRunProgress, cancellation_token}, hardware::on_board_led::OnBoardLed};

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

pub static CURRENT_RUN_PROGRESS: Mutex<Option<flume::Sender<AutoScriptRunProgress>>> =
    Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn delay(exec_env: *mut WASMExecEnv, milliseconds: u8) {
    check_cancellation!(exec_env);
    thread::sleep(Duration::from_millis(milliseconds as u64));
}

#[unsafe(no_mangle)]
pub extern "C" fn print(exec_env: *mut WASMExecEnv, text: u32) {
    check_cancellation!(exec_env);
    {
        let current_run_progress = CURRENT_RUN_PROGRESS.lock().unwrap();

        if let Some(current_run_progress) = current_run_progress.as_ref() {
            if let Err(_err) = current_run_progress.send(AutoScriptRunProgress::Log{message: "hi".to_string()}) { // TODO: this shouldn't block maybe?
                //
            }
        }
    }

    info!("received: {:?}", text);
}

#[unsafe(no_mangle)]
pub extern "C" fn set_onboard_led_color(exec_env: *mut WASMExecEnv, r: u8, g: u8, b: u8) {
    check_cancellation!(exec_env);
    OnBoardLed::set_color(RGB8 { r, g, b })
}
