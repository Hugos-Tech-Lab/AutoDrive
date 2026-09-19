use std::{
    ffi::c_void,
    sync::mpsc::SyncSender,
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::inter_thread::{self, InterThreadListener, InterThreadProducer};
use anyhow::{Result, bail};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_sync::{channel::Channel, mutex::Mutex};
use esp_idf_sys::{MALLOC_CAP_8BIT, esp_get_free_heap_size, heap_caps_get_largest_free_block};
use flume::Sender;
use log::info;
use serde::Serialize;
use wamr_rust_sdk::{function::Function, instance::Instance, module::Module, runtime::Runtime};
pub mod cancellation_token;
pub mod exposed_functions;
use static_cell::StaticCell;

#[derive(Debug)]
pub enum WasmResponse {
    SuccessfullyInstalled,
    SuccessfullyRun,
    InstallError(String),
    StartError(String),
    InternalError(String),
}

#[derive(Serialize)]
pub enum AutoScriptRunProgress {
    Starting,
    Log,
    Stopping,
}

#[derive(PartialEq)]
pub enum AutoScriptState {
    Uninstalled,
    Installed,
    Running,
}

pub struct AutoScript {
    wasm_thread: JoinHandle<()>,
    auto_script_state: Mutex<CriticalSectionRawMutex, AutoScriptState>,
    producer: InterThreadProducer<WasmThreadCommand, WasmResponse>,
}

impl AutoScript {
    pub fn new() -> Self {
        let (producer, listener) = inter_thread::create::<WasmThreadCommand, WasmResponse>();
        let wasm_thread = thread::Builder::new()
            .name("wasm".to_owned())
            .stack_size(34 * 1024) // by decreasing the stack size
            .spawn({
                move || {
                    loop {
                        match WasmThread::listen(&listener) {
                            Ok(ok) => {
                                log::info!("wasm thread is stopping")
                            }
                            Err(err) => {
                                log::warn!(
                                    "wasm thread crashed: '{:?}'. Restarting in 1 second",
                                    err
                                );
                                thread::sleep(Duration::from_secs(1));
                            }
                        }
                    }
                }
            })
            .unwrap(); // TODO: remove unwrap

        Self {
            producer,
            wasm_thread,
            auto_script_state: AutoScriptState::Uninstalled.into(),
        }
    }

    pub async fn install(&self, data: Vec<u8>) -> Result<WasmResponse> {
        {
            let auto_script_state_guard = self.auto_script_state.lock().await;
            if *auto_script_state_guard == AutoScriptState::Running {
                return Ok(WasmResponse::StartError("already running".to_string()));
            }
        }

        let res = self
            .producer
            .send_async(WasmThreadCommand::Install { data })
            .await; // TODO: add progress here too

        {
            let mut auto_script_state_guard = self.auto_script_state.lock().await;
            *auto_script_state_guard = AutoScriptState::Installed;
        }

        Ok(res)
    }

    pub async fn run(&self, progress: Sender<AutoScriptRunProgress>) -> Result<WasmResponse> {
        {
            let mut auto_script_state_guard = self.auto_script_state.lock().await;
            if *auto_script_state_guard == AutoScriptState::Running {
                return Ok(WasmResponse::StartError("already running".to_string()));
            }
            if *auto_script_state_guard == AutoScriptState::Uninstalled {
                return Ok(WasmResponse::StartError("not installed".to_string()));
            }
            *auto_script_state_guard = AutoScriptState::Running;
        }

        let res = self
            .producer
            .send_async(WasmThreadCommand::Run { progress })
            .await;
        {
            let mut auto_script_state_guard = self.auto_script_state.lock().await; // TODO: only set running when the response is actually ok
            *auto_script_state_guard = AutoScriptState::Running;
        }
        Ok(res)
    }

    pub fn cancel(&self) {
        cancellation_token::cancel();
    }
}

pub enum WasmThreadCommand {
    Install {
        data: Vec<u8>,
    },
    Run {
        progress: Sender<AutoScriptRunProgress>,
    },
}

struct WasmThread;

impl WasmThread {
    fn build_runtime() -> Runtime {
        let runtime = Runtime::builder()
            .use_system_allocator()
            .register_host_function(
                "delay",
                crate::auto_script::exposed_functions::delay as *mut c_void,
            )
            .register_host_function(
                "print",
                crate::auto_script::exposed_functions::print as *mut c_void,
            )
            .register_host_function(
                "set_onboard_led_color",
                crate::auto_script::exposed_functions::set_onboard_led_color as *mut c_void,
            )
            .build()
            .unwrap();
        runtime
    }

    #[allow(
        unused_assignments,
        reason = "it seems the compiler is used how we are using objects through references..."
    )]
    fn listen(listener: &InterThreadListener<WasmThreadCommand, WasmResponse>) -> Result<()> {
        let runtime = Self::build_runtime();
        let mut maybe_module = None;
        let mut maybe_instance = None;
        let mut maybe_main_function = None;
        info!("WASM runtime started");

        while let Ok((command, response)) = listener.listen() {
            // TODO: remove unwrap
            match command {
                WasmThreadCommand::Install { data } => {
                    maybe_main_function = None; // when commenting this line, the compiler doesn't complain... isn't that a memory bug?
                    maybe_instance = None;
                    maybe_module = None;

                    let free = unsafe { esp_get_free_heap_size() };
                    let largest = unsafe { heap_caps_get_largest_free_block(MALLOC_CAP_8BIT) };

                    info!("Free heap: {} bytes", free);
                    info!("Largest free block: {} bytes", largest);

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
                        )))?;
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
                        )))?;
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
                        )))?;
                        maybe_main_function = None;
                        maybe_instance = None;
                        maybe_module = None;
                        continue;
                    }
                    info!("g");

                    response.send(WasmResponse::SuccessfullyInstalled)?;
                }
                WasmThreadCommand::Run { progress } => {
                    info!("running");
                    progress.send(AutoScriptRunProgress::Starting)?;

                    let Some(ref instance) = maybe_instance else {
                        // response.send(WasmResponse::StartError(
                        //     "Wasm instance not installed. Try to install first before running"
                        //         .to_string(),
                        // ))?;
                        continue;
                    };
                    cancellation_token::reset();

                    progress.send(AutoScriptRunProgress::Starting).unwrap();
                    if let Some(ref main_function) = maybe_main_function {
                        progress.send(AutoScriptRunProgress::Starting)?;
                        let res = main_function.call(&instance, &vec![]);
                        // response.send(WasmResponse::SuccessfullyRun)?;
                    } else {
                        response.send(WasmResponse::StartError(
                            "Main function not found. Try to reinstall before running".to_string(),
                        ))?;
                        continue;
                    }
                    response.send(WasmResponse::SuccessfullyRun)?;
                }
            }
        }

        info!("WASM thread stopped");
        Ok(())
    }
}
