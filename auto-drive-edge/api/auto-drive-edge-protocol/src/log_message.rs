use crate::auto_drive_edge_service::v1::{self, LogLevel, LogSystemOrigin};

impl From<auto_drive_common_types::LogMessage> for v1::LogMessage {
    fn from(value: auto_drive_common_types::LogMessage) -> Self {
        Self {
            system_origin: LogSystemOrigin::from(value.system_origin).into(),
            level: LogLevel::from(value.level).into(),
            message: value.message,
        }
    }
}

impl TryFrom<v1::LogMessage> for auto_drive_common_types::LogMessage {
    type Error = ();

    fn try_from(value: v1::LogMessage) -> Result<Self, Self::Error> {
        let system_origin = v1::LogSystemOrigin::try_from(value.system_origin).map_err(|_| ())?;
        let system_origin = auto_drive_common_types::LogSystemOrigin::try_from(system_origin)?;

        let level = v1::LogLevel::try_from(value.level).map_err(|_| ())?;
        let level = auto_drive_common_types::LogLevel::try_from(level)?;

        Ok(Self {
            system_origin,
            level,
            target: "".to_string(),
            message: value.message,
            run_uuid: None,
        })
    }
}
