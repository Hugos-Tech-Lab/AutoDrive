use esp_idf_svc::hal::delay::FreeRtos;
use log::info;
use wamr_rust_sdk::sys::{
    WASMExecEnv, wasm_runtime_addr_app_to_native, wasm_runtime_get_custom_data,
    wasm_runtime_get_module_inst, wasm_runtime_terminate, wasm_runtime_validate_app_addr,
};

use crate::auto_script::wasm_thread::WasmData;

pub fn terminate(exec_env: *mut WASMExecEnv) {
    if exec_env.is_null() {
        return;
    }
    let instance = unsafe { wasm_runtime_get_module_inst(exec_env) };
    unsafe { wasm_runtime_terminate(instance) };
}

#[unsafe(no_mangle)]
pub extern "C" fn delay(exec_env: *mut WASMExecEnv, milliseconds: u8) {
    let wasm_data: &mut WasmData = get_wasm_data(exec_env);
    if wasm_data.ct.is_cancelled() {
        terminate(exec_env);
    }

    FreeRtos::delay_ms(milliseconds as u32);
}

#[unsafe(no_mangle)]
pub extern "C" fn print(exec_env: *mut WASMExecEnv, ptr: u32, len: u32) {
    let wasm_data: &mut WasmData = get_wasm_data(exec_env);
    if wasm_data.ct.is_cancelled() {
        terminate(exec_env);
    }

    let mut value = "";
    unsafe {
        let module_inst = wasm_runtime_get_module_inst(exec_env);

        // wasm_runtime_set_user_data(exec_env, user_data);

        let native_ptr = wasm_runtime_validate_app_addr(module_inst, ptr as u64, len as u64);

        if !native_ptr {
            return;
        }

        let native_ptr = wasm_runtime_addr_app_to_native(module_inst, ptr as u64);

        if native_ptr.is_null() {
            return;
        }

        let slice = core::slice::from_raw_parts(native_ptr as *const u8, len as usize);

        match core::str::from_utf8(slice) {
            Ok(text) => value = text,
            Err(_) => log::error!("<invalid UTF-8>"),
        }
    }

    info!("{}", value.to_string());
}

#[unsafe(no_mangle)]
pub extern "C" fn set_onboard_led_color(exec_env: *mut WASMExecEnv, r: u8, g: u8, b: u8) {
    let wasm_data: &mut WasmData = get_wasm_data(exec_env);
    if wasm_data.ct.is_cancelled() {
        terminate(exec_env);
    }

    unimplemented!();
}

#[unsafe(no_mangle)]
pub extern "C" fn set_motor_a_speed(exec_env: *mut WASMExecEnv, speed: i8) {
    let wasm_data: &mut WasmData = get_wasm_data(exec_env);
    if wasm_data.ct.is_cancelled() {
        terminate(exec_env);
    }

    let mut hardware = wasm_data.hardware.lock().unwrap();
    hardware.set_wheel_a_speed(speed, crate::hardware::RequestOrigin::Manual);
}

#[unsafe(no_mangle)]
pub extern "C" fn set_motor_b_speed(exec_env: *mut WASMExecEnv, speed: i8) {
    let wasm_data: &mut WasmData = get_wasm_data(exec_env);
    if wasm_data.ct.is_cancelled() {
        terminate(exec_env);
    }

    let mut hardware = wasm_data.hardware.lock().unwrap();
    hardware.set_wheel_b_speed(speed, crate::hardware::RequestOrigin::Manual);
}

#[unsafe(no_mangle)]
pub extern "C" fn get_ir_utmost_front_utmost_left_value(exec_env: *mut WASMExecEnv) -> u16 {
    let wasm_data: &mut WasmData = get_wasm_data(exec_env);
    if wasm_data.ct.is_cancelled() {
        terminate(exec_env);
    }

    let hardware = wasm_data.hardware.lock().unwrap();
    hardware.get_ir_utmost_front_utmost_left_value()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_ir_utmost_front_utmost_right_value(exec_env: *mut WASMExecEnv) -> u16 {
    let wasm_data: &mut WasmData = get_wasm_data(exec_env);
    if wasm_data.ct.is_cancelled() {
        terminate(exec_env);
    }

    let hardware = wasm_data.hardware.lock().unwrap();
    hardware.get_ir_utmost_front_utmost_right_value()
}

fn get_wasm_data<'a>(exec_env: *mut WASMExecEnv) -> &'a mut WasmData {
    //  TODO: disable WAMR_BUILD_LIB_PTHREAD because this is not thread safe
    unsafe {
        let module_inst = wasm_runtime_get_module_inst(exec_env);
        let custom_data = wasm_runtime_get_custom_data(module_inst);

        &mut *(custom_data as *mut WasmData)
    }
}
