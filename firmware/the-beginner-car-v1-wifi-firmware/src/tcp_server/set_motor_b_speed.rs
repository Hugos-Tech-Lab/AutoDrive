use std::{sync::mpsc::SyncSender, time::Duration};

// use auto_drive_interfaces::the_beginner_car::hardware_mcu::{RequestToHardware, ResponseFromHardware};
use esp_idf_svc::http::server::Request;
use the_beginner_car_spi_protocol::{RequestToHardware, ResponseFromHardware};

use crate::{HardwareMessage, tcp_server::error::HttpServerError};

pub fn get_speed(
    req: &mut Request<&mut esp_idf_svc::http::server::EspHttpConnection<'_>>,
) -> Result<i8, HttpServerError> {
    let mut buf = [0u8; 4];
    let n = req
        .read(&mut buf)
        .map_err(|a| HttpServerError::UserError(a.to_string()))?;

    let speed: i8 = match std::str::from_utf8(&buf[..n])
        .ok()
        .and_then(|s| s.trim().parse::<i8>().ok())
    {
        Some(speed) => speed,
        None => {
            return Err(HttpServerError::UserError(
                "Invalid motor speed".to_string(),
            ));
        }
    };
    Ok(speed)
}

pub fn set_motor_b_speed_on_hardware(
    hardware_sender: SyncSender<HardwareMessage>,
    speed: i8,
) -> Result<Vec<u8>, HttpServerError> {
    let (sender, receiver) =
        std::sync::mpsc::sync_channel::<Result<ResponseFromHardware, anyhow::Error>>(1);

    if let Err(e) = hardware_sender.send(HardwareMessage {
        request: RequestToHardware::SetMotorBSpeed(speed),
        response_tx: sender,
    }) {
        return Err(HttpServerError::InternalProgrammerError(format!(
            "Failed to send motor command: {e:?}"
        )));
    }

    let receive = match receiver.recv_timeout(Duration::from_secs(1)) {
        Ok(Ok(response)) => response,
        Ok(Err(e)) => {
            return Err(HttpServerError::InternalProgrammerError(format!(
                "Hardware error: {e:?}"
            )));
        }

        Err(e) => {
            return Err(HttpServerError::InternalProgrammerError(format!(
                "Hardware response timeout/disconnect: {e:?}"
            )));
        }
    };

    let body = match serde_json::to_vec(&receive) {
        Ok(body) => body,
        Err(e) => {
            return Err(HttpServerError::InternalProgrammerError(format!(
                "Failed to serialize hardware response: {e:?}"
            )));
        }
    };

    Ok(body)
}
