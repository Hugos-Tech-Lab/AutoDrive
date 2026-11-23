use auto_drive_common_types::LogLevel;

use crate::auto_drive_edge_service::v1;

impl From<LogLevel> for v1::LogLevel {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Error => Self::Error,
            LogLevel::Warn => Self::Warn,
            LogLevel::Info => Self::Info,
            LogLevel::Debug => Self::Debug,
            LogLevel::Trace => Self::Trace,
        }
    }
}

impl TryFrom<v1::LogLevel> for LogLevel {
    type Error = ();

    fn try_from(value: v1::LogLevel) -> Result<Self, ()> {
        match value {
            v1::LogLevel::Unspecified => Err(()),
            v1::LogLevel::Error => Ok(Self::Error),
            v1::LogLevel::Warn => Ok(Self::Warn),
            v1::LogLevel::Info => Ok(Self::Info),
            v1::LogLevel::Debug => Ok(Self::Debug),
            v1::LogLevel::Trace => Ok(Self::Trace),
        }
    }
}
