use std::{ffi::c_void, thread};

use anyhow::Error;
use log::info;
use wamr_rust_sdk::{function::Function, instance::Instance, module::Module, runtime::Runtime};

use crate::hardware::Hardware;

#[derive(PartialEq)]
enum DeviceMode {
    Manual,
    Automatic,
}

#[unsafe(no_mangle)]
extern "C" fn control_wheel(wheel: f32, speed: f32) {
    println!("[WASM] wheel = {}", wheel);
    println!("[WASM] speed = {}", speed);
}

pub struct DeviceControl<'runtime> {
    hardware: Hardware,
    // runtime: Runtime,
    auto_execution: Option<Module<'runtime>>,
    mode: DeviceMode,
}

const BASIC_WASM: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/basic.wasm"));

impl<'runtime> DeviceControl<'runtime> {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            hardware: Hardware {},
            auto_execution: None,
            mode: DeviceMode::Manual,
        })
    }

    pub fn activate_auto(&self) {
        let thread = thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(move || {
                let runtime = Runtime::builder()
                    .use_system_allocator()
                    .register_host_function(
                        "delay",
                        crate::wasm::exposed_functions::delay as *mut c_void,
                    )
                    .register_host_function(
                        "print",
                        crate::wasm::exposed_functions::print as *mut c_void,
                    )
                    .register_host_function(
                        "set_onboard_led_color",
                        crate::wasm::exposed_functions::set_onboard_led_color as *mut c_void,
                    )
                    .build()
                    .unwrap();


                let module = Module::from_vec(&runtime, BASIC_WASM.to_vec(), "env").unwrap();
                let instance = Instance::new(&runtime, &module, 1024 * 32).unwrap();
                let function = Function::find_export_func(&instance, "main").unwrap();
                let params = vec![];

                info!("starting to run");

                let result = function.call(&instance, &params); // Change call to call_pthread
                let res = result.unwrap();
                info!("{:?}", res);

            })
            .unwrap()
            .join();
    }

    pub fn control_wheel_manual(&self) {
        // let runtime = Runtime::new()?;

        // let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        // d.push("gcd_wasm32_wasi.wasm");
        // let mut module = Module::from_file(&runtime, d.as_path())?;

        if self.mode == DeviceMode::Automatic {
            // TODO: return error
        }

        //
        //
    }

    pub fn control_wheel_automatic(&self) {
        //
    }

    //
}
