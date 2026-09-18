use std::{
    ffi::c_void,
    thread::{self, JoinHandle},
};

use anyhow::Result;
use log::info;
use wamr_rust_sdk::{function::Function, instance::Instance, module::Module, runtime::Runtime};

use crate::inter_thread::{self, InterThreadListener, InterThreadProducer, InterThreadResponse};

pub mod cancellation_token;
pub mod exposed_functions;

pub enum WasmResponse {
    Success,
    Error(String),
}

pub enum WasmState {
    Uninstalled,
    Stopped,
    Running,
}
pub struct Wasm {
    wasm_thread: JoinHandle<()>,
    producer: InterThreadProducer<WasmThreadCommand, WasmResponse>,
}

impl Wasm {
    pub fn new() -> Self {
        let (producer, listener) = inter_thread::create::<WasmThreadCommand, WasmResponse>();
        let wasm_thread = thread::Builder::new()
            .name("wasm".to_owned())
            .stack_size(64 * 1024)
            .spawn({
                move || {
                    let res = WasmThread::listen(listener); // TODO: something about this res
                }
            })
            .unwrap(); // TODO: remove unwrap

        Self {
            producer,
            wasm_thread,
        }
    }

    pub fn install(&self) {
        //
    }

    pub fn run(&self) {
        //
    }

    pub fn cancel(&self) {
        cancellation_token::cancel();
    }
}

pub enum WasmThreadCommand {
    Install { data: Vec<u8> },
    Start,
}

struct WasmThread;

impl WasmThread {
    fn build_runtime() -> Runtime {
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
        runtime
    }

    fn listen(listener: InterThreadListener<WasmThreadCommand, WasmResponse>) -> Result<()> {
        let runtime = Self::build_runtime();
        #[allow(unused_assignments, reason = "using it indirectly - compiler cannot see")]
        let mut maybe_module = None;
        let mut maybe_instance = None;
        let mut maybe_main_function = None;
        info!("WASM runtime started");

        // let mut executable = Option::<WasmExecutable::<'runtime, 'module>>::None;
        while let Ok((command, mut response)) = listener.listen() {
            match command {
                WasmThreadCommand::Install { data } => {
                    maybe_main_function = None;
                    maybe_instance = None;
                    maybe_module = None;

                    let module = maybe_module.insert(
                        Module::from_vec(&runtime, data, "env")
                            .map_err(|e| format!("failed to create module: {e:?}"))
                            .unwrap(),
                    );

                    maybe_instance = Some(
                        Instance::new(&runtime, module, 1024 * 32)
                            .map_err(|e| format!("failed to create instance: {e:?}"))
                            .unwrap(),
                    );

                    // Unfortunately Rust doesn't seem to provide a way to get immutable references on insert.
                    let instance = maybe_instance.as_ref().unwrap();

                    maybe_main_function = Some(
                        Function::find_export_func(instance, "main")
                            .map_err(|e| {
                                format!("failed to find export function (is main defined?): {e:?}")
                            })
                            .unwrap(),
                    );

                    response.send(WasmResponse::Success);
                }
                WasmThreadCommand::Start => {
                    let Some(ref instance) = maybe_instance else {
                        continue;
                    };

                    if let Some(ref main_function) = maybe_main_function {
                        main_function.call(&instance, &vec![]).unwrap();
                    } else {
                        continue;
                    }
                }
            }
        }

        info!("WASM thread stopped");
        Ok(())
    }
}
