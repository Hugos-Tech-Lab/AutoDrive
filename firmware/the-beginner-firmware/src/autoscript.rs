// use crate::wasm::{Wasm, WasmResponse};
// use anyhow::Result;
// use flume::{Receiver, Sender};
// use serde::Serialize;
// use std::sync::mpsc::{self, SyncSender};

// enum AutoScriptState {
//     Uninstalled,
//     Ready,
//     Running,
// }

// #[derive(Serialize)]
// pub enum AutoScriptRunProgress {
//     Starting,
//     Log,
//     Stopping,
// }

// pub struct AutoScript {
//     state: AutoScriptState,
//     wasm: Wasm,
// }

// impl AutoScript {
//     pub fn new(wasm: Wasm) -> Self {
//         Self {
//             state: AutoScriptState::Uninstalled,
//             wasm,
//         }
//     }

//     pub async fn install(&self, data: Vec<u8>) -> anyhow::Result<WasmResponse> {
//         self.wasm.install(data).await
//     }

//     pub async fn run(&self, progress: Sender<AutoScriptRunProgress>) -> Result<WasmResponse> {
//         self.wasm.run(progress).await
//     }

//     pub fn cancel(&self) {
//         self.wasm.cancel();
//     }
// }
