use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, SyncSender},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    auto_script::{
        cancellation_token::CancellationToken,
        wasm_thread::{WasmThread, WasmThreadCommand},
    },
    hardware::TheBeginnerCarHardware,
};
use anyhow::Result;
use log::info;
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
    pub fn new(the_beginner_car: Arc<Mutex<TheBeginnerCarHardware>>) -> Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let ct = CancellationToken::new();

        let wasm_thread = thread::Builder::new()
            .name("wasm".to_owned())
            .stack_size(8 * 1024)
            .spawn({
                let ct = ct.clone();
                move || {
                    let mut auto_script_wasm =
                        WasmThread::new(the_beginner_car, ct).expect("Failed to init WasmThread");

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
            running: Arc::new(Mutex::new(false)),
            ct,
        })
    }

    pub fn run(&self, data: Vec<u8>) -> Result<()> {
        let mut running_guard = self.running.lock().unwrap();
        if *running_guard == true {
            panic!("already running");
        }

        self.ct.reset();

        let (sender, receiver) = mpsc::sync_channel(1);
        self.sender
            .send((WasmThreadCommand::Run { data, running: self.running.clone() }, sender))
            .unwrap(); // TODO: wait for response
        receiver.recv_timeout(Duration::from_secs(5))??;

        *running_guard = true;

        Ok(())
    }

    pub fn cancel(&self) {
        self.ct.cancel();
    }
}
