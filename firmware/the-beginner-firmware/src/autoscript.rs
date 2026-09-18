use std::sync::mpsc;

use crate::wasm::{Wasm, WasmResponse};

enum AutoScriptState {
    Uninstalled,
    Ready,
    Running,
}

pub struct AutoScript {
    state: AutoScriptState,
    wasm: Wasm,
}

impl AutoScript {
    pub fn new(wasm: Wasm) -> Self {
        Self {
            state: AutoScriptState::Uninstalled,
            wasm,
        }
    }

    pub fn install(&self, data: Vec<u8>) -> anyhow::Result<WasmResponse> {
        let (sender, receiver) = mpsc::channel::<WasmResponse>(); // TODO: maybe use spsc

        // self.wasm.send(WasmCommand::Install { data, response: sender });

        // let response = receiver.recv().unwrap(); // TODO: unwrap + is recv ok like this?
        Ok(WasmResponse::Success)
    }

    pub fn cancel(&self) {}
}
