use std::sync::Arc;

use edge_http::io::Error;
use embedded_io_async::{Read, Write};
use futures::{FutureExt, select};
use log::info;

use crate::{
    auto_script::{AutoScript, AutoScriptRunProgress},
    hardware::on_board_led::OnBoardLed,
    http_server::{CORS_HEADERS, FirmwareUpdate200Response},
};

pub async fn set_motor_a_speed<T, const N: usize>(
    conn: &mut edge_http::io::server::Connection<'_, T, N>,
) -> Result<(), Error<T::Error>>
where
    T: Read + Write,
{
    // let headers = conn.headers()?;

    let mut body = [0u8; 4];

    let n = conn.read(&mut body).await?;

    let speed: i8 = core::str::from_utf8(&body[..n])
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);

    log::info!("Setting motor A speed to {}", speed);

    OnBoardLed::set_motor_a_speed(speed);

    let body = serde_json::to_vec(&FirmwareUpdate200Response {
        status: "ok".to_string(),
    })
    .unwrap();

    let mut res_headers = CORS_HEADERS.to_vec();
    res_headers.push(("Content-Type", "application/json"));

    conn.initiate_response(200, Some("OK"), &res_headers)
        .await?;

    conn.write_all(&body).await?;

    Ok(())
}


pub async fn set_motor_b_speed<T, const N: usize>(
    conn: &mut edge_http::io::server::Connection<'_, T, N>,
) -> Result<(), Error<T::Error>>
where
    T: Read + Write,
{
    // let headers = conn.headers()?;

    let mut body = [0u8; 4];

    let n = conn.read(&mut body).await?;

    let speed: i8 = core::str::from_utf8(&body[..n])
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);

    log::info!("Setting motor B speed to {}", speed);

    OnBoardLed::set_motor_b_speed(speed);

    let body = serde_json::to_vec(&FirmwareUpdate200Response {
        status: "ok".to_string(),
    })
    .unwrap();

    let mut res_headers = CORS_HEADERS.to_vec();
    res_headers.push(("Content-Type", "application/json"));

    conn.initiate_response(200, Some("OK"), &res_headers)
        .await?;

    conn.write_all(&body).await?;

    Ok(())
}