use std::{
    ffi::c_void,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use log::info;
use wamr_rust_sdk::{
    function::Function,
    instance::Instance,
    module::Module,
    runtime::Runtime,
};

pub mod exposed_functions;

pub enum WasmCommand {
    Install {
        data: Vec<u8>,
        response: Sender<WasmResponse>,
    },
}

pub enum WasmResponse {
    Success,
    Error(String),
}

pub struct Wasm {
    sender: Sender<WasmCommand>,
}

impl Wasm {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<WasmCommand>();

        thread::Builder::new()
            .name("wasm".to_owned())
            .stack_size(64 * 1024)
            .spawn(move || {
                Self::run(receiver);
            })
            .unwrap();

        Self { sender }
    }

    pub fn send(&self, wasm_command: WasmCommand) {
        self.sender.send(wasm_command); // TODO: error handling here
    }

    fn run(receiver: Receiver<WasmCommand>) {
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

        info!("WASM runtime started");

        while let Ok(command) = receiver.recv() {
            match command {
                WasmCommand::Install { data, response } => {
                    let result = Self::install(&runtime, data);

                    let ret = match result {
                        Ok(()) => WasmResponse::Success,
                        Err(err) => WasmResponse::Error(err),
                    };

                    let _ = response.send(ret);
                }
            }
        }

        info!("WASM thread stopped");
    }

    fn install(runtime: &Runtime, data: Vec<u8>) -> Result<(), String> {
        info!("Installing WASM: {} bytes", data.len());

        let module = Module::from_vec(runtime, data, "env")
            .map_err(|e| format!("failed to create module: {e:?}"))?;

        let instance = Instance::new(runtime, &module, 1024 * 32)
            .map_err(|e| format!("failed to create instance: {e:?}"))?;

        let function = Function::find_export_func(&instance, "main")
            .map_err(|e| format!("failed to find export function (is main defined?): {e:?}"))?;

        let params = vec![];

        info!("starting to run");

        function
            .call(&instance, &params)
            .map_err(|e| format!("WASM execution failed: {e:?}"))?;

        Ok(())
    }
}
