use std::ffi::c_void;

use esp_idf_svc::hal::delay::FreeRtos;
use log::info;
use the_beginner_car_wasm_sdk::host::WasmExports;
use wamr_rust_sdk::sys::{
    WASMExecEnv, wasm_runtime_addr_app_to_native, wasm_runtime_get_custom_data,
    wasm_runtime_get_module_inst, wasm_runtime_terminate, wasm_runtime_validate_app_addr,
};

use crate::auto_script::wasm_thread::WasmData;

pub struct TheBeginnerCarExports;

impl WasmExports for TheBeginnerCarExports {
    /// # Safety
    ///
    /// Must be called from within a WASM runtime with a valid exec_env.
    #[unsafe(no_mangle)]
    unsafe extern "C" fn delay(exec_env: *mut c_void, milliseconds: u8) {
        if exec_env.is_null() {
            return;
        }
        let exec_env = exec_env.cast::<WASMExecEnv>();

        let wasm_data: &mut WasmData = get_wasm_data(exec_env);
        if wasm_data.ct.is_cancelled() {
            unsafe { terminate(exec_env) };
        }

        FreeRtos::delay_ms(milliseconds as u32);
    }

    /// # Safety
    ///
    /// Must be called from within a WASM runtime with a valid exec_env.
    #[unsafe(no_mangle)]
    unsafe extern "C" fn print(exec_env: *mut c_void, ptr: u32, len: u32) {
        if exec_env.is_null() {
            return;
        }
        let exec_env = exec_env.cast::<WASMExecEnv>();

        let wasm_data: &mut WasmData = get_wasm_data(exec_env);
        if wasm_data.ct.is_cancelled() {
            unsafe { terminate(exec_env) };
        }

        let mut value = "";
        unsafe {
            let module_inst = wasm_runtime_get_module_inst(exec_env);

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

        info!("{}", value);
    }

    /// # Safety
    ///
    /// Must be called from within a WASM runtime with a valid exec_env.
    #[unsafe(no_mangle)]
    unsafe extern "C" fn set_onboard_led_color(exec_env: *mut c_void, _r: u8, _g: u8, _b: u8) {
        if exec_env.is_null() {
            return;
        }
        let exec_env = exec_env.cast::<WASMExecEnv>();
        let wasm_data: &mut WasmData = get_wasm_data(exec_env);
        if wasm_data.ct.is_cancelled() {
            unsafe { terminate(exec_env) };
        }

        unimplemented!();
    }

    /// # Safety
    ///
    /// Must be called from within a WASM runtime with a valid exec_env.
    #[unsafe(no_mangle)]
    unsafe extern "C" fn set_motor_a_speed(exec_env: *mut c_void, speed: i8) {
        if exec_env.is_null() {
            return;
        }
        let exec_env = exec_env.cast::<WASMExecEnv>();
        let wasm_data: &mut WasmData = get_wasm_data(exec_env);
        if wasm_data.ct.is_cancelled() {
            unsafe { terminate(exec_env) };
        }

        let mut hardware = wasm_data.hardware.lock().unwrap();
        hardware.set_wheel_a_speed(speed, crate::hardware::RequestOrigin::Manual);
    }

    /// # Safety
    ///
    /// Must be called from within a WASM runtime with a valid exec_env.
    #[unsafe(no_mangle)]
    unsafe extern "C" fn set_motor_b_speed(exec_env: *mut c_void, speed: i8) {
        if exec_env.is_null() {
            return;
        }
        let exec_env = exec_env.cast::<WASMExecEnv>();
        let wasm_data: &mut WasmData = get_wasm_data(exec_env);
        if wasm_data.ct.is_cancelled() {
            unsafe { terminate(exec_env) };
        }

        let mut hardware = wasm_data.hardware.lock().unwrap();
        hardware.set_wheel_b_speed(speed, crate::hardware::RequestOrigin::Manual);
    }

    /// # Safety
    ///
    /// Must be called from within a WASM runtime with a valid exec_env.
    #[unsafe(no_mangle)]
    unsafe extern "C" fn get_ir_utmost_front_utmost_left_value(exec_env: *mut c_void) -> u16 {
        if exec_env.is_null() {
            return 0;
        }
        let exec_env = exec_env.cast::<WASMExecEnv>();
        let exec_env = exec_env.cast::<WASMExecEnv>();
        let wasm_data: &mut WasmData = get_wasm_data(exec_env);
        if wasm_data.ct.is_cancelled() {
            unsafe { terminate(exec_env) };
        }

        let hardware = wasm_data.hardware.lock().unwrap();
        hardware.get_ir_utmost_front_utmost_left_value()
    }

    /// # Safety
    ///
    /// Must be called from within a WASM runtime with a valid exec_env.
    #[unsafe(no_mangle)]
    unsafe extern "C" fn get_ir_utmost_front_utmost_right_value(exec_env: *mut c_void) -> u16 {
        if exec_env.is_null() {
            return 0;
        }
        let exec_env = exec_env.cast::<WASMExecEnv>();
        let wasm_data: &mut WasmData = get_wasm_data(exec_env);
        if wasm_data.ct.is_cancelled() {
            unsafe { terminate(exec_env) };
        }

        let hardware = wasm_data.hardware.lock().unwrap();
        hardware.get_ir_utmost_front_utmost_right_value()
    }
    
    unsafe extern "C" fn set_pin_left_on(exec_env: *mut c_void) {
        if exec_env.is_null() {
            return;
        }
        let exec_env = exec_env.cast::<WASMExecEnv>();
        let wasm_data: &mut WasmData = get_wasm_data(exec_env);
        if wasm_data.ct.is_cancelled() {
            unsafe { terminate(exec_env) };
        }


        let mut hardware = wasm_data.hardware.lock().unwrap();
        hardware.led_left().on().unwrap();
    }
    
    unsafe extern "C" fn set_pin_left_off(exec_env: *mut c_void) {
        if exec_env.is_null() {
            return;
        }
        let exec_env = exec_env.cast::<WASMExecEnv>();
        let wasm_data: &mut WasmData = get_wasm_data(exec_env);
        if wasm_data.ct.is_cancelled() {
            unsafe { terminate(exec_env) };
        }

        let mut hardware = wasm_data.hardware.lock().unwrap();
        hardware.led_left().on().unwrap();
    }
    
    unsafe extern "C" fn set_pin_right_on(exec_env: *mut c_void) {
        if exec_env.is_null() {
            return;
        }
        let exec_env = exec_env.cast::<WASMExecEnv>();
        let wasm_data: &mut WasmData = get_wasm_data(exec_env);
        if wasm_data.ct.is_cancelled() {
            unsafe { terminate(exec_env) };
        }

        let mut hardware = wasm_data.hardware.lock().unwrap();
        hardware.led_right().on().unwrap();
    }
    
    unsafe extern "C" fn set_pin_right_off(exec_env: *mut c_void) {
        if exec_env.is_null() {
            return;
        }
        let exec_env = exec_env.cast::<WASMExecEnv>();
        let wasm_data: &mut WasmData = get_wasm_data(exec_env);
        if wasm_data.ct.is_cancelled() {
            unsafe { terminate(exec_env) };
        }

        let mut hardware = wasm_data.hardware.lock().unwrap();
        hardware.led_right().off().unwrap();
    }
}

unsafe fn terminate(exec_env: *mut WASMExecEnv) {
    if exec_env.is_null() {
        return;
    }
    let instance = unsafe { wasm_runtime_get_module_inst(exec_env) };
    unsafe { wasm_runtime_terminate(instance) };
}

fn get_wasm_data<'a>(exec_env: *mut WASMExecEnv) -> &'a mut WasmData {
    //  TODO: disable WAMR_BUILD_LIB_PTHREAD because this is not thread safe
    unsafe {
        let module_inst = wasm_runtime_get_module_inst(exec_env);
        let custom_data = wasm_runtime_get_custom_data(module_inst);

        &mut *(custom_data as *mut WasmData)
    }
}
