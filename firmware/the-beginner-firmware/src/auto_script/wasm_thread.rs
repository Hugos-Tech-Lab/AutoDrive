use std::{
    ffi::c_void,
    sync::mpsc::SyncSender,
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{auto_script::{AutoScriptRunProgress, WasmResponse}, inter_thread::{self, InterThreadListener, InterThreadProducer}};
use anyhow::{Result, bail};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_sync::{channel::Channel, mutex::Mutex};
use esp_idf_sys::{MALLOC_CAP_8BIT, esp_get_free_heap_size, heap_caps_get_largest_free_block};
use flume::Sender;
use log::info;
use serde::Serialize;
use wamr_rust_sdk::{function::Function, instance::Instance, module::Module, runtime::Runtime};

pub enum WasmThreadCommand {
    Install {
        data: Vec<u8>,
    },
    Run {
        progress: Sender<AutoScriptRunProgress>,
    },
}

pub struct WasmThread;

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
    pub fn listen(listener: &InterThreadListener<WasmThreadCommand, WasmResponse>) -> Result<()> {
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
                    crate::auto_script::cancellation_token::reset();

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