use esp_idf_svc::http::{Method, server::EspHttpServer};

pub fn set_handles(server: &mut EspHttpServer) -> anyhow::Result<()> {
    server.fn_handler("/autoscript/info", Method::Options, |req| -> anyhow::Result<()> {
        req.into_response(
            204,
            None,
            &[
                ("Access-Control-Allow-Origin", "*"),
                ("Access-Control-Allow-Methods", "GET, OPTIONS"),
                ("Access-Control-Allow-Headers", "Content-Type"),
            ],
        )?;
        Ok(())
    })?;
    // server.fn_handler("/firmware/info", Method::Get, move |req| -> anyhow::Result<()> {
    //     let esp_ota = esp_ota.lock().unwrap();
    //     let device = get_device_info(&esp_ota)?;

    //     let body = serde_json::to_vec(&device)?;

    //     let mut response = req.into_response(
    //         200,
    //         None,
    //         &[
    //             ("Content-Type", "application/json"),
    //             ("Access-Control-Allow-Origin", "*"),
    //         ],
    //     )?;

    //     response.write(&body)?;

    //     Ok(())
    // })?;

    Ok(())
}