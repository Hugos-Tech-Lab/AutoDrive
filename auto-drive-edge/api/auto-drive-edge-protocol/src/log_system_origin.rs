use auto_drive_common_types::LogSystemOrigin;

use crate::auto_drive_edge_service::v1;

impl From<LogSystemOrigin> for v1::LogSystemOrigin {
    fn from(value: LogSystemOrigin) -> Self {
        match value {
            LogSystemOrigin::EdgeServer => Self::WasmUnspecified,
            LogSystemOrigin::WasmMcu => Self::WasmMcu,
            LogSystemOrigin::WifiMcu => Self::WifiMcu,
        }
    }
}

impl TryFrom<v1::LogSystemOrigin> for LogSystemOrigin {
    type Error = ();

    fn try_from(value: v1::LogSystemOrigin) -> Result<Self, Self::Error> {
        match value {
            v1::LogSystemOrigin::WasmUnspecified => Err(()),
            v1::LogSystemOrigin::WasmMcu => Ok(Self::WasmMcu),
            v1::LogSystemOrigin::WifiMcu => Ok(Self::WifiMcu),
        }
    }
}
