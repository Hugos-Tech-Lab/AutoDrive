use esp_idf_svc::{
    http::{Method, server::EspHttpServer},
};

use crate::logger::LOG_BUFFER;

pub fn set_handles(server: &mut EspHttpServer) -> anyhow::Result<()> {
    server.fn_handler("/firmware/logs", Method::Get, |req| -> anyhow::Result<()> {
        let logs = if let Some(buffer) = LOG_BUFFER.get() {
            let buffer = buffer
                .lock()
                .map_err(|_| anyhow::anyhow!("log buffer poisoned"))?;

            buffer.iter().cloned().collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        let body = serde_json::to_vec(&logs)?;

        let mut response = req.into_response(
            200,
            None,
            &[
                ("Content-Type", "application/json"),
                ("Access-Control-Allow-Origin", "*"),
            ],
        )?;

        response.write(&body)?;

        Ok(())
    })?;

    Ok(())
}
