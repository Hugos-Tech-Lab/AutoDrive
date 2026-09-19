use std::{
    ffi::c_void, sync::mpsc::SyncSender, thread::{self, JoinHandle},
};

use anyhow::{Result, bail};
use log::info;
use wamr_rust_sdk::{function::Function, instance::Instance, module::Module, runtime::Runtime};

use crate::{autoscript::AutoScriptRunProgress, inter_thread::{self, InterThreadListener, InterThreadProducer}};

pub mod cancellation_token;
pub mod exposed_functions;

pub enum WasmResponse {
    SuccessfullyInstalled,
    SuccessfullyRun,
    InstallError(String),
    StartError(String),
    InternalError(String),
}

#[derive(PartialEq)]
pub enum WasmState {
    Uninstalled,
    Installed,
    Running,
}

pub struct Wasm {
    wasm_thread: JoinHandle<()>,
    wasm_state: WasmState,
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
            wasm_state: WasmState::Uninstalled,
        }
    }

    pub fn install(&self, data: Vec<u8>) -> Result<WasmResponse> {
        let res = self.producer.send(WasmThreadCommand::Install { data }); // TODO: add progress here too
        // self.wasm_state = WasmState::Installed;
        Ok(res)
    }

    pub fn run(&self, progress: SyncSender<AutoScriptRunProgress>) -> Result<WasmResponse> {
        if self.wasm_state == WasmState::Running {
            bail!("already running");
        }

        // self.wasm_state = WasmState::Running;
        let res = self.producer.send(WasmThreadCommand::Run { progress });
        // self.wasm_state = WasmState::Installed;

        Ok(res)
    }

    pub fn cancel(&self) {
        cancellation_token::cancel();
    }
}

pub enum WasmThreadCommand {
    Install { data: Vec<u8> },
    Run { progress: SyncSender<AutoScriptRunProgress> }
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

    #[allow(
        unused_assignments,
        reason = "it seems the compiler is used how we are using objects through references..."
    )]
    fn listen(listener: InterThreadListener<WasmThreadCommand, WasmResponse>) -> Result<()> {
        let runtime = Self::build_runtime();
        let mut maybe_module = None;
        let mut maybe_instance = None;
        let mut maybe_main_function = None;
        info!("WASM runtime started");

        while let Ok((command, mut response)) = listener.listen() {
            match command {
                WasmThreadCommand::Install { data } => {
                    maybe_main_function = None; // when commenting this line, the compiler doesn't complain... isn't that a memory bug?
                    maybe_instance = None;
                    maybe_module = None;

                    info!("a");
                    let module = maybe_module.insert(
                        Module::from_vec(&runtime, data, "env")
                            .map_err(|e| format!("failed to create module: {e:?}"))
                            .unwrap(),
                    );
                    info!("b");

                    let mut instance_create_error = Option::None;
                    match Instance::new(&runtime, module, 1024 * 32) {
                        Ok(instance) => maybe_instance = Some(instance),
                        Err(error) => instance_create_error = Some(error),
                    }
                    info!("c");

                    if let Some(instance_create_error) = instance_create_error {
                        response.send(WasmResponse::InstallError(format!(
                            "failed to create instance: {}",
                            instance_create_error
                        )));
                        maybe_main_function = None;
                        maybe_instance = None;
                        maybe_module = None;
                        continue;
                    }
                    info!("d");

                    // Unfortunately Rust doesn't seem to provide a way to get immutable references on insert.
                    let Some(instance) = maybe_instance.as_ref() else {
                        response.send(WasmResponse::InternalError(format!(
                            "failed to get the created instance. This is a code bug in the library"
                        )));
                        maybe_main_function = None;
                        maybe_instance = None;
                        maybe_module = None;
                        continue;
                    };
                    info!("e");

                    let mut main_function_find_error = Option::None;
                    match Function::find_export_func(instance, "main") {
                        Ok(main_function) => maybe_main_function = Some(main_function),
                        Err(error) => main_function_find_error = Some(error),
                    }
                    info!("f");

                    if let Some(main_function_find_error) = main_function_find_error {
                        response.send(WasmResponse::InstallError(format!(
                            "failed to find export function (is main defined?): {}",
                            main_function_find_error
                        )));
                        maybe_main_function = None;
                        maybe_instance = None;
                        maybe_module = None;
                        continue;
                    }
                    info!("g");

                    response.send(WasmResponse::SuccessfullyInstalled);
                }
                WasmThreadCommand::Run { progress } => {
                    let Some(ref instance) = maybe_instance else {
                        response.send(WasmResponse::StartError(
                            "Wasm instance not installed. Try to install first before running"
                                .to_string(),
                        ));
                        continue;
                    };
                    cancellation_token::reset();

                    progress.send(AutoScriptRunProgress::Starting).unwrap();
                    progress.send(AutoScriptRunProgress::Starting).unwrap();
                    progress.send(AutoScriptRunProgress::Starting).unwrap();
                    progress.send(AutoScriptRunProgress::Starting).unwrap();
                    if let Some(ref main_function) = maybe_main_function {
                        progress.send(AutoScriptRunProgress::Starting);
                        let res = main_function.call(&instance, &vec![]);
                    } else {
                        response.send(WasmResponse::StartError(
                            "Main function not found. Try to reinstall before running".to_string(),
                        ));
                        continue;
                    }
                    response.send(WasmResponse::SuccessfullyRun);
                }
            }
        }

        info!("WASM thread stopped");
        Ok(())
    }
}
