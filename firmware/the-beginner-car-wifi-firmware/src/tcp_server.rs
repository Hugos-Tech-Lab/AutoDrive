use std::sync::mpsc::SyncSender;
use std::time::Duration;

pub mod auto_script;
pub mod error;
pub mod info;
pub mod logs;
pub mod set_motor_a_speed;
pub mod set_motor_b_speed;
pub mod update;
pub mod verify_and_set_valid;

use esp_idf_svc::http::Method;
use esp_idf_svc::http::server::{Configuration, EspHttpConnection, EspHttpServer, Request};
use esp_idf_svc::io::Write;

use crate::HardwareMessage;

static CORS_HEADERS: [(&str, &str); 3] = [
    ("Access-Control-Allow-Origin", "*"),
    ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
    ("Access-Control-Allow-Headers", "Content-Type"),
];

/// Starts the HTTP server and routes the endpoints.
/// Note: You MUST store the returned `EspHttpServer` in your `main()` function
/// so it doesn't drop out of scope. If it drops, the server stops.
pub fn run(
    hardware_sender: SyncSender<HardwareMessage>,
) -> Result<EspHttpServer<'static>, anyhow::Error> {
    // We boost the stack size to 10k to safely handle JSON parsing and larger strings
    let config = Configuration {
        stack_size: 10240,
        max_sessions: 4,
        ..Default::default()
    };

    let mut server = EspHttpServer::new(&config)?;
    log::info!("Running native ESP HTTP server on port 80");

    // --- OPTIONS / CORS Preflight Routes ---
    // In esp-idf-svc, we can reuse this handler for all OPTIONS routes
    let options_handler = |req: Request<&mut esp_idf_svc::http::server::EspHttpConnection>| -> Result<(), anyhow::Error> {
        req.into_response(204, None, &CORS_HEADERS)?;
        Ok(())
    };

    server.fn_handler("/autoscript/upload", Method::Options, options_handler)?;
    server.fn_handler("/autoscript/run", Method::Options, options_handler)?;
    server.fn_handler(
        "/hardware/set_motor_a_speed",
        Method::Options,
        options_handler,
    )?;
    server.fn_handler(
        "/hardware/set_motor_b_speed",
        Method::Options,
        options_handler,
    )?;
    server.fn_handler("/logs", Method::Options, options_handler)?;
    server.fn_handler("/hardware/set_motor_a_speed", Method::Post, {
        let hardware_sender = hardware_sender.clone();
        move |mut req| -> Result<(), anyhow::Error> {
            let speed = set_motor_a_speed::get_speed(&mut req);
            let speed = match speed {
                Ok(speed) => speed,
                Err(err) => return handle_http_error(req, err),
            };

            let speed =
                set_motor_a_speed::set_motor_a_speed_on_hardware(hardware_sender.clone(), speed);
            let res = match speed {
                Ok(speed) => speed,
                Err(err) => return handle_http_error(req, err),
            };

            let mut response = req.into_response(200, Some("OK"), &CORS_HEADERS)?;
            response.write_all(&res)?;

            Ok(())
        }
    })?;

    server.fn_handler("/hardware/set_motor_b_speed", Method::Post, {
        let hardware_sender = hardware_sender.clone();
        move |mut req| -> Result<(), anyhow::Error> {
            let speed = set_motor_b_speed::get_speed(&mut req);
            let speed = match speed {
                Ok(speed) => speed,
                Err(err) => return handle_http_error(req, err),
            };

            let speed =
                set_motor_b_speed::set_motor_b_speed_on_hardware(hardware_sender.clone(), speed);
            let res = match speed {
                Ok(speed) => speed,
                Err(err) => return handle_http_error(req, err),
            };

            let mut response = req.into_response(200, Some("OK"), &CORS_HEADERS)?;
            response.write_all(&res)?;

            Ok(())
        }
    })?;

    // GET /logs
    server.fn_handler(
        "/logs",
        Method::Get,
        move |mut req| -> Result<(), anyhow::Error> {
            logs::logs(req)?;
            Ok(())
        },
    )?;

    // GET /coredump
    server.fn_handler(
        "/coredump",
        Method::Get,
        move |mut req| -> Result<(), anyhow::Error> {
            // You will need to update `logs::coredump` to accept `Request<&mut EspHttpConnection>`
            // logs::coredump(req)?;

            let mut response = req.into_response(200, Some("OK"), &CORS_HEADERS)?;
            response.write_all(b"Coredump output")?;
            Ok(())
        },
    )?;

    // --- Method Not Allowed / 405 Handling ---
    // Note: EspHttpServer handles 404 (Not Found) automatically for unregistered paths.
    // If you explicitly want 405s for specific paths with wrong methods, you map them like this:
    let method_not_allowed = |req: Request<&mut esp_idf_svc::http::server::EspHttpConnection>| -> Result<(), anyhow::Error> {
        req.into_response(405, Some("Method Not Allowed"), &CORS_HEADERS)?;
        Ok(())
    };

    server.fn_handler("/autoscript/upload", Method::Get, method_not_allowed)?;
    server.fn_handler("/autoscript/upload", Method::Post, method_not_allowed)?;
    server.fn_handler("/autoscript/run", Method::Get, method_not_allowed)?;
    server.fn_handler("/autoscript/cancel", Method::Get, method_not_allowed)?;

    Ok(server)
}

fn handle_http_error(
    req: Request<&mut EspHttpConnection<'_>>,
    err: error::HttpServerError,
) -> Result<(), anyhow::Error> {
    let (status, reason) = match &err {
        error::HttpServerError::UserError(_) => (400, "Bad Request"),
        error::HttpServerError::InternalProgrammerError(_) => (500, "Internal Server Error"),
    };

    if let error::HttpServerError::InternalProgrammerError(err) = &err {
        log::error!("Internal server error: {err:?}");
    }

    let body = match err {
        error::HttpServerError::UserError(err) => {
            format!("User error: {:?}", err)
        }
        error::HttpServerError::InternalProgrammerError(err) => {
            format!("Internal programmer error: {:?}", err)
        }
    };

    let mut response = req.into_response(status, Some(reason), &CORS_HEADERS)?;

    response.write_all(body.as_bytes())?;

    Ok(())
}
