use std::{ffi::c_void, thread};

use anyhow::Error;
use log::info;
use wamr_rust_sdk::{function::Function, instance::Instance, module::Module, runtime::Runtime};

use crate::hardware::Hardware;

// #[derive(PartialEq)]
// enum DeviceMode {
//     Manual,
//     Automatic,
// }

pub struct DeviceControl<'runtime> {
    hardware: Hardware,
    // runtime: Runtime,
    auto_execution: Option<Module<'runtime>>,
    // mode: DeviceMode,
}

const BASIC_WASM: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/basic.wasm"));

impl<'runtime> DeviceControl<'runtime> {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            hardware: Hardware {},
            auto_execution: None,
            // mode: DeviceMode::Manual,
        })
    }

    pub fn activate_auto(&self) {

    }

    // pub fn control_wheel_manual(&self) {
    //     // let runtime = Runtime::new()?;

    //     // let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    //     // d.push("gcd_wasm32_wasi.wasm");
    //     // let mut module = Module::from_file(&runtime, d.as_path())?;

    //     if self.mode == DeviceMode::Automatic {
    //         // TODO: return error
    //     }

    //     //
    //     //
    // }

    // pub fn control_wheel_automatic(&self) {
    //     //
    // }

    //
}
