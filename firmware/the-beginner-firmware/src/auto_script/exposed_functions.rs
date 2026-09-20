use std::{sync::Mutex, thread, time::Duration};

use flume::Sender;
use log::info;
use smart_leds_trait::RGB8;
use wamr_rust_sdk::sys::{WASMExecEnv, wasm_runtime_addr_app_to_native, wasm_runtime_get_module_inst, wasm_runtime_terminate, wasm_runtime_validate_app_addr};

use crate::{
    auto_script::{AutoScriptRunProgress, cancellation_token},
    hardware::on_board_led::OnBoardLed,
};

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
pub extern "C" fn print(
    exec_env: *mut WASMExecEnv,
    ptr: u32,
    len: u32,
) {
    let mut value = "";
    unsafe {
        let module_inst = wasm_runtime_get_module_inst(exec_env);

        let native_ptr = wasm_runtime_validate_app_addr(
            module_inst,
            ptr as u64,
            len as u64,
        );

        if !native_ptr {
            return;
        }

        let native_ptr =
            wasm_runtime_addr_app_to_native(module_inst, ptr as u64);

        if native_ptr.is_null() {
            return;
        }

        let slice = core::slice::from_raw_parts(
            native_ptr as *const u8,
            len as usize,
        );

        match core::str::from_utf8(slice) {
            Ok(text) => value = text,
            Err(_) => log::error!("<invalid UTF-8>"),
        }
    }


    check_cancellation!(exec_env);
    {
        let current_run_progress = CURRENT_RUN_PROGRESS.lock().unwrap();

        if let Some(current_run_progress) = current_run_progress.as_ref() {
            if let Err(_err) = current_run_progress.send(AutoScriptRunProgress::Log {
                message: value.to_string(),
            }) { // TODO: this shouldn't block maybe?
                //
            }
        }
    }

    info!("{:?}", value);
}

#[unsafe(no_mangle)]
pub extern "C" fn set_onboard_led_color(exec_env: *mut WASMExecEnv, r: u8, g: u8, b: u8) {
    check_cancellation!(exec_env);
    OnBoardLed::set_color(RGB8 { r, g, b })
}
