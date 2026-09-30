use edge_http::io::Error;
use embedded_io_async::{Read, Write};
use crate::http_server::CORS_HEADERS;

use crate::{
    logger::LOG_BUFFER,
};

pub async fn logs<T, const N: usize>(
    conn: &mut edge_http::io::server::Connection<'_, T, N>,
) -> Result<(), Error<T::Error>>
where
    T: Read + Write,
{
    let logs = if let Some(buffer) = LOG_BUFFER.get() {
        let buffer = buffer
            .lock()
            .map_err(|_| Error::ConnectionClosed)?;

        buffer.iter().cloned().collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let body = serde_json::to_vec(&logs).unwrap();

    let mut res_headers = CORS_HEADERS.to_vec();
    res_headers.push(("Content-Type", "application/json"));

    conn.initiate_response(200, Some("OK"), &res_headers)
        .await?;

    conn.write_all(&body).await?;

    Ok(())
}

const CHUNK_SIZE: usize = 1024;

pub async fn coredump<T, const N: usize>(
    conn: &mut edge_http::io::server::Connection<'_, T, N>,
) -> Result<(), Error<T::Error>>
where
    T: Read + Write,
{
    let mut address: usize = 0;
    let mut size: usize = 0;

    let err = unsafe {
        esp_idf_sys::esp_core_dump_image_get(
            &mut address as *mut usize,
            &mut size as *mut usize,
        )
    };

    if err != esp_idf_sys::ESP_OK {
        let mut headers = CORS_HEADERS.to_vec();
        headers.push(("Content-Type", "text/plain"));

        conn.initiate_response(404, Some("No Coredump"), &headers)
            .await?;

        conn.write_all(b"No coredump available").await?;

        return Ok(());
    }

    let mut headers = CORS_HEADERS.to_vec();
    headers.push(("Content-Type", "application/octet-stream"));
    headers.push((
        "Content-Disposition",
        "attachment; filename=\"coredump.bin\"",
    ));

    conn.initiate_response(200, Some("OK"), &headers)
        .await?;

    let mut buffer = [0u8; CHUNK_SIZE];
    let mut offset = 0usize;

    while offset < size {
        let remaining = size - offset;
        let chunk_size = remaining.min(CHUNK_SIZE);

        let err = unsafe {
            esp_idf_sys::esp_flash_read(
                esp_idf_sys::esp_flash_default_chip,
                buffer.as_mut_ptr() as *mut core::ffi::c_void,
                (address + offset) as u32,
                chunk_size as u32,
            )
        };

        if err != esp_idf_sys::ESP_OK {
            // Connection is already in progress, so just abort it.
            return Err(Error::ConnectionClosed);
        }

        conn.write_all(&buffer[..chunk_size]).await?;

        offset += chunk_size;
    }

    Ok(())
}
