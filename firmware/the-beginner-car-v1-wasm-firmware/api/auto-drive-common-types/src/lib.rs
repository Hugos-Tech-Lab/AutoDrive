use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LogSystemOrigin {
    CloudServer,
    EdgeServer,
    WasmMcu,
    WifiMcu,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LogMessage {
    pub system_origin: LogSystemOrigin,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub trace_id: Option<String>,
}
