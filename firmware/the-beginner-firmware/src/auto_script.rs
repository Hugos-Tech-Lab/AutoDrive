use std::{
    ffi::c_void,
    sync::mpsc::SyncSender,
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{auto_script::wasm_thread::{WasmThread, WasmThreadCommand}, inter_thread::{self, InterThreadListener, InterThreadProducer}};
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
pub mod wasm_thread;
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


