use std::{
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    auto_script::{
        cancellation_token::CancellationToken,
        wasm_thread::{WasmThread, WasmThreadCommand},
    },
    inter_thread::{self, InterThreadProducer},
};
use anyhow::{Result, bail};
use flume::Sender;
use log::{info, warn};
use serde::Serialize;

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

#[derive(Serialize)]
pub enum AutoScriptRunProgress {
    ReceivedRunAction,
    InstantiatingInstance,
    FindingMain,
    CallingMain,
    Log { message: String },
    Stopping,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum AutoScriptState {
    Uninstalled = 0,
    Installed = 1,
    Running = 2,
}

// Helper to convert atomic reads back into our Enum safely
impl From<u8> for AutoScriptState {
    fn from(val: u8) -> Self {
        match val {
            1 => AutoScriptState::Installed,
            2 => AutoScriptState::Running,
            _ => AutoScriptState::Uninstalled,
        }
    }
}

pub struct AutoScript {
    wasm_thread: JoinHandle<()>,
    state: Arc<AtomicU8>,
    producer: InterThreadProducer<WasmThreadCommand, Result<()>>,
    ct: CancellationToken,
}

impl AutoScript {
    pub fn new() -> Result<Self> {
        let (producer, listener) = inter_thread::create::<WasmThreadCommand, Result<()>>();
        let ct = CancellationToken::new();

        let wasm_thread = thread::Builder::new()
            .name("wasm".to_owned())
            .stack_size(8 * 1024)
            .spawn({
                let ct = ct.clone();
                move || {
                    let mut auto_script_wasm =
                        WasmThread::new(ct).expect("Failed to init WasmThread");

                    loop {
                        match auto_script_wasm.listen(&listener) {
                            Ok(_) => {
                                info!("wasm thread processed command successfully");
                            }
                            Err(err) => {
                                warn!("wasm thread crashed: '{:?}'. Restarting in 1s", err);
                                thread::sleep(Duration::from_secs(1));
                            }
                        }
                    }
                }
            })?;

        Ok(Self {
            producer,
            wasm_thread,
            state: Arc::new(AtomicU8::new(AutoScriptState::Uninstalled as u8)),
            ct,
        })
    }

    pub fn get_state(&self) -> AutoScriptState {
        self.state.load(Ordering::Acquire).into()
    }

    pub async fn install(&self, data: Vec<u8>) -> Result<()> {
        if self.get_state() == AutoScriptState::Running {
            bail!("already running");
        }

        self.producer
            .send_async(WasmThreadCommand::Install { data })
            .await?;

        // Update to installed ONLY if send_async succeeded
        self.state
            .store(AutoScriptState::Installed as u8, Ordering::Release);

        Ok(())
    }

    pub async fn run(&self, progress: Sender<AutoScriptRunProgress>) -> Result<()> {
        // Atomic compare_exchange ensures we can ONLY transition from Installed -> Running.
        // This completely eliminates race conditions where two requests call run() at once.
        self.state
            .compare_exchange(
                AutoScriptState::Installed as u8,
                AutoScriptState::Running as u8,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .map_err(|_| anyhow::anyhow!("Cannot run: not installed or already running"))?;


        let _guard = RunGuard { state: &self.state, ct: self.ct.clone() };

        self.ct.reset();

        self.producer
            .send_async(WasmThreadCommand::Run { progress })
            .await?;
        Ok(())
    }

    pub fn cancel(&self) {
        self.ct.cancel();
    }
}

/// A scope guard that automatically handles state cleanup
struct RunGuard<'a> {
    state: &'a AtomicU8,
    ct: CancellationToken,
}

impl<'a> Drop for RunGuard<'a> {
    fn drop(&mut self) {
        // If this future drops unexpectedly (e.g. HTTP disconnect), trigger a cancel
        self.ct.cancel();        

        // Safely transition back to Installed ONLY if we were Running.
        let _ = self.state.compare_exchange(
            AutoScriptState::Running as u8,
            AutoScriptState::Installed as u8,
            Ordering::SeqCst,
            Ordering::SeqCst,
        );
    }
}
