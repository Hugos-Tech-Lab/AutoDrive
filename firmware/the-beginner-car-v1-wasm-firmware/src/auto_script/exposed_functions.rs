use std::ffi::c_void;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_sys::xTaskGetTickCount;
use log::info;
use std::sync::atomic::Ordering;
use the_beginner_car_wasm_sdk::host::WasmExports;
use wamr_rust_sdk::sys::{
    WASMExecEnv, wasm_runtime_addr_app_to_native, wasm_runtime_get_custom_data,
    wasm_runtime_get_module_inst, wasm_runtime_terminate, wasm_runtime_validate_app_addr,
};

use crate::auto_script::wasm_thread::WasmData;

macro_rules! wasm_hardware_lock {
    ($exec_env:expr) => {{
        if $exec_env.is_null() {
            panic!("exec_env is null, we're about to crash!");
        }
        let exec_env = $exec_env.cast::<WASMExecEnv>();
        let wasm_data: &mut WasmData = get_wasm_data(exec_env);
        wasm_data
            .last_host_fn_call_ticks_ms
            .store(unsafe { xTaskGetTickCount() }, Ordering::Relaxed);

        if wasm_data.ct.is_cancelled() {
            unsafe { terminate(exec_env) };
        }

        wasm_data
    }};
}

pub struct TheBeginnerCarExports;

impl WasmExports for TheBeginnerCarExports {
    /// # Safety
    ///
    /// Must be called from within a WASM runtime with a valid exec_env.
    #[unsafe(no_mangle)]
    unsafe extern "C" fn delay(exec_env: *mut c_void, milliseconds: u8) {
        let _ = wasm_hardware_lock!(exec_env);
        FreeRtos::delay_ms(milliseconds as u32);
    }

    /// # Safety
    ///
    /// Must be called from within a WASM runtime with a valid exec_env.
    #[unsafe(no_mangle)]
    unsafe extern "C" fn print(exec_env: *mut c_void, ptr: u32, len: u32) {
        let _ = wasm_hardware_lock!(exec_env);
        let exec_env = exec_env.cast::<WASMExecEnv>();

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
        let _ = wasm_hardware_lock!(exec_env);
        unimplemented!();
    }

    /// # Safety
    ///
    /// Must be called from within a WASM runtime with a valid exec_env.
    #[unsafe(no_mangle)]
    unsafe extern "C" fn set_motor_left_speed(exec_env: *mut c_void, speed: i8) {
        let mut hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();

        hardware.set_wheel_left_speed(speed);
    }

    /// # Safety
    ///
    /// Must be called from within a WASM runtime with a valid exec_env.
    #[unsafe(no_mangle)]
    unsafe extern "C" fn set_motor_right_speed(exec_env: *mut c_void, speed: i8) {
        let mut hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware.set_wheel_right_speed(speed);
    }

    unsafe extern "C" fn set_led_left_on(exec_env: *mut c_void) {
        let _ = wasm_hardware_lock!(exec_env);
    }

    unsafe extern "C" fn set_led_left_off(exec_env: *mut c_void) {
        let _ = wasm_hardware_lock!(exec_env);
    }

    unsafe extern "C" fn set_led_right_on(exec_env: *mut c_void) {
        let _ = wasm_hardware_lock!(exec_env);
    }

    unsafe extern "C" fn set_led_right_off(exec_env: *mut c_void) {
        let _ = wasm_hardware_lock!(exec_env);
    }

    unsafe extern "C" fn get_ir_back_center_value(exec_env: *mut c_void) -> u16 {
        let hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware.get_ir_back_center_value()
    }

    unsafe extern "C" fn get_ir_front_left_value(exec_env: *mut c_void) -> u16 {
        let hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware.get_ir_front_left_value()
    }

    unsafe extern "C" fn get_ir_front_center_value(exec_env: *mut c_void) -> u16 {
        let hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware.get_ir_front_center_value()
    }

    unsafe extern "C" fn get_ir_front_right_value(exec_env: *mut c_void) -> u16 {
        let hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware.get_ir_front_right_value()
    }

    unsafe extern "C" fn get_ir_front_far_left_value(exec_env: *mut c_void) -> u16 {
        let hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware.get_ir_front_far_left_value()
    }

    unsafe extern "C" fn get_ir_front_far_right_value(exec_env: *mut c_void) -> u16 {
        let hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware.get_ir_front_far_right_value()
    }

    unsafe extern "C" fn get_ir_back_left_value(exec_env: *mut c_void) -> u16 {
        let hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware.get_ir_back_left_value()
    }

    unsafe extern "C" fn set_motor_left_pwm(exec_env: *mut c_void, pwm: i8) {
        let mut hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        let _ = hardware.wheel_left().motor().set_pwm(pwm);
    }

    unsafe extern "C" fn set_motor_left_rpm(exec_env: *mut c_void, rpm: i32) {
        let mut hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware.wheel_left().motor().set_rpm(rpm);
    }

    unsafe extern "C" fn get_wheel_left_circumference_cm(exec_env: *mut c_void) -> f32 {
        let mut hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware
            .wheel_left()
            .tachometer()
            .get_wheel_circumference_cm()
    }

    unsafe extern "C" fn get_wheel_left_pulses_per_rotation(exec_env: *mut c_void) -> u32 {
        let mut hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware.wheel_left().tachometer().get_pulses_per_rotation()
    }

    unsafe extern "C" fn get_wheel_left_total_pulses(exec_env: *mut c_void) -> i32 {
        let mut hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware.wheel_left().tachometer().get_total_pulses()
    }

    unsafe extern "C" fn set_motor_right_pwm(exec_env: *mut c_void, pwm: i8) {
        let mut hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware.wheel_right().motor().set_pwm(pwm);
    }

    unsafe extern "C" fn set_motor_right_rpm(exec_env: *mut c_void, rpm: i32) {
        let mut hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware.wheel_right().motor().set_rpm(rpm);
    }

    unsafe extern "C" fn get_wheel_right_circumference_cm(exec_env: *mut c_void) -> f32 {
        let mut hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware
            .wheel_right()
            .tachometer()
            .get_wheel_circumference_cm()
    }

    unsafe extern "C" fn get_wheel_right_pulses_per_rotation(exec_env: *mut c_void) -> u32 {
        let mut hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware
            .wheel_right()
            .tachometer()
            .get_pulses_per_rotation()
    }

    unsafe extern "C" fn get_wheel_right_total_pulses(exec_env: *mut c_void) -> i32 {
        let mut hardware = wasm_hardware_lock!(exec_env).hardware.lock().unwrap();
        hardware.wheel_right().tachometer().get_total_pulses()
    }

    unsafe extern "C" fn set_config_recommended_pwm(exec_env: *mut c_void, rpm: i16, pwm: i8) {
        todo!()
    }

    unsafe extern "C" fn get_config_recommended_pwm(exec_env: *mut c_void, rpm: i16) -> i8 {
        todo!()
    }

    unsafe extern "C" fn get_config_recommended_pos_min_rpm(exec_env: *mut c_void) -> i16 {
        todo!()
    }

    unsafe extern "C" fn get_config_recommended_pos_max_rpm(exec_env: *mut c_void) -> i16 {
        todo!()
    }

    unsafe extern "C" fn get_config_recommended_neg_min_rpm(exec_env: *mut c_void) -> i16 {
        todo!()
    }

    unsafe extern "C" fn get_config_recommended_neg_max_rpm(exec_env: *mut c_void) -> i16 {
        todo!()
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
