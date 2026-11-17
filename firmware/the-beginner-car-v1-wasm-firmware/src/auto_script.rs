use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, AtomicUsize, Ordering},
        mpsc::{self, SyncSender},
    },
    thread::JoinHandle,
    time::Duration,
};

use crate::{
    auto_script::{
        cancellation_token::CancellationToken,
        wasm_thread::{WasmThread, WasmThreadCommand},
    },
    spawn_thread::spawn_thread,
    the_beginner_car_hardware::TheBeginnerCarHardware,
    uptime_in_ms::uptime_in_ms,
};
use anyhow::{Result, bail};
use esp_idf_svc::hal::delay::FreeRtos;
use log::{info, warn};
use wamr_rust_sdk::sys::{WASMModuleInstanceCommon, wasm_runtime_terminate};
pub mod cancellation_token;
pub mod exposed_functions;
pub mod wasm_thread;

#[derive(Debug)]
pub enum WasmResponse {
    SuccessfullyInstalled,
    SuccessfullyRun,
    InstallError(String),
    StartError(String),
    InternalError(String),
}

pub struct AutoScript {
    _wasm_thread: JoinHandle<()>,
    running: Arc<Mutex<bool>>,
    sender: SyncSender<(WasmThreadCommand, SyncSender<anyhow::Result<()>>)>,
    ct: CancellationToken,
}

impl AutoScript {
    pub fn new(
        the_beginner_car: Arc<TheBeginnerCarHardware>,
        on_stopped_running: impl Fn() + Send + 'static,
    ) -> Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let ct = CancellationToken::new();
        let running = Arc::new(Mutex::new(false));
        let last_host_fn_call_ticks_ms = Arc::new(AtomicU32::new(uptime_in_ms()));
        let active_wasm_instance = Arc::new(AtomicUsize::new(0));

        spawn_thread(c"wasm_supervisor", 6, 2_000, {
            let last_host_fn_call_ticks_ms = last_host_fn_call_ticks_ms.clone();
            let active_wasm_instance = active_wasm_instance.clone();
            move || {
                loop {
                    let current_ticks_ms = uptime_in_ms();
                    let last_host_fn_call_ticks_ms =
                        last_host_fn_call_ticks_ms.load(Ordering::Relaxed);
                    let time_since_last_host_fn_call_ms =
                        current_ticks_ms.wrapping_sub(last_host_fn_call_ticks_ms);

                    if time_since_last_host_fn_call_ms > 2000 {
                        let inst_ptr = active_wasm_instance.load(Ordering::Relaxed);
                        if inst_ptr != 0 {
                            warn!("Timeout! Terminate runtime");
                            unsafe {
                                wasm_runtime_terminate(inst_ptr as *mut WASMModuleInstanceCommon)
                            };
                        }
                    }
                    FreeRtos::delay_ms(1000);
                }
            }
        })
        .unwrap();

        let wasm_thread = spawn_thread(c"wasm", 5, 8 * 1024, {
            let ct = ct.clone();
            let running = running.clone();

            move || {
                let mut auto_script_wasm = WasmThread::new(
                    the_beginner_car,
                    active_wasm_instance,
                    move || {
                        {
                            let mut running_guard = running.lock().unwrap();
                            *running_guard = false;
                        }

                        on_stopped_running();
                    },
                    last_host_fn_call_ticks_ms,
                    ct,
                )
                .expect("Failed to init WasmThread");

                loop {
                    match auto_script_wasm.listen(&receiver) {
                        Ok(_) => {
                            info!("wasm thread processed command successfully");
                        }
                        Err(err) => {
                            panic!("wasm thread crashed: '{:?}'", err);
                        }
                    }
                }
            }
        })?;

        Ok(Self {
            sender,
            _wasm_thread: wasm_thread,
            running,
            ct,
        })
    }

    pub fn run(&self, data: Vec<u8>) -> Result<()> {
        let mut running_guard = self.running.lock().unwrap();
        if *running_guard {
            bail!("already running");
        }

        self.ct.reset();

        let (sender, receiver) = mpsc::sync_channel(1);
        self.sender
            .send((WasmThreadCommand::Run { data }, sender))
            .unwrap(); // TODO: wait for response
        receiver.recv_timeout(Duration::from_secs(5))??;

        *running_guard = true;

        Ok(())
    }

    pub fn cancel(&self) {
        self.ct.cancel();
    }

    pub fn is_running(&self) -> bool {
        let running_guard = self.running.lock().unwrap();
        *running_guard
    }
}
