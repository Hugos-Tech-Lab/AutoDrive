use std::sync::Arc;
use std::sync::mpsc::{Receiver, SyncSender};
use std::time::Duration;

pub mod auto_script;
pub mod error;
pub mod info;
pub mod logs;
pub mod set_motor_a_speed;
pub mod set_motor_b_speed;
pub mod update;
pub mod verify_and_set_valid;

use async_io::Async;
use esp_idf_svc::http::Method;
use esp_idf_svc::http::server::{Configuration, EspHttpConnection, EspHttpServer, Request};
use esp_idf_svc::io::Write;
use futures::{AsyncReadExt, AsyncWriteExt};
use log::error;
use log::info;
use the_beginner_car_tcp_protocol::TheBeginnerCarIncomingMessages;
use std::{io, thread};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use futures::executor::LocalSpawner;
use futures::task::LocalSpawnExt;

use crate::HardwareMessage;

async fn handle(mut stream: Async<TcpStream>, hardware_sender: SyncSender<HardwareMessage>) {
    loop {
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await.unwrap();
        let len = u32::from_be_bytes(len_bytes) as usize;
        if len > 1024 {
            panic!("message too large: {len} bytes");
        }

        let mut bytes = vec![0; len];
        match stream.read_exact(&mut bytes).await {
            Ok(_) => {
                let message: TheBeginnerCarIncomingMessages = postcard::from_bytes(&bytes).unwrap();

                match message {
                    TheBeginnerCarIncomingMessages::SetMotorASpeed { speed } => {
                        let _ = set_motor_a_speed::set_motor_a_speed_on_hardware(hardware_sender.clone(), speed).unwrap();
                        println!("SetMotorASpeed: {:?}", speed)
                    }
                    TheBeginnerCarIncomingMessages::SetMotorBSpeed { speed } => {
                        let _ = set_motor_b_speed::set_motor_b_speed_on_hardware(hardware_sender.clone(), speed).unwrap();
                        println!("SetMotorBSpeed: {:?}", speed)
                    }
                    TheBeginnerCarIncomingMessages::InformPositionFromCamera { x, y } => {
                        println!("InformPositionFromCamera: {:?} {:?}", x, y)
                    }
                    TheBeginnerCarIncomingMessages::InstallWasm { bytes } => {
                        println!("InstallWasm: {:?}", bytes)
                    }
                    TheBeginnerCarIncomingMessages::RunWasm => println!("RunWasm"),
                    TheBeginnerCarIncomingMessages::GetBatteryReading => {
                        // let voltage = volt.recv().unwrap();
                        // let message = TheBeginnerCarOutgoingMessages::BatteryReading { voltage };
                        // let bytes = postcard::to_stdvec(&message).unwrap();
                        // let len = u32::try_from(bytes.len()).unwrap();

                        // // Send [4-byte length][postcard message]
                        // stream.write_all(&len.to_be_bytes()).await.unwrap();
                        // stream.write_all(&bytes).await.unwrap();

                        // dbg!("wrote");

                        // stream.write(buf)

                    },
                }
            }
            Err(err) => {
                panic!("{}", err);
            }
        }

        thread::sleep(Duration::from_secs(1));
    }
}

pub async fn accept(spawner: LocalSpawner, hardware_sender: SyncSender<HardwareMessage>) -> Result<(), io::Error> {
    info!("About to bind a simple echo service to port 8080; do `telnet <ip-from-above>:8080`");

    let addr = "0.0.0.0:8080".to_socket_addrs()?.next().unwrap();
    let listener = Async::<TcpListener>::bind(addr)?;

    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((stream, addr)) => {
                info!("Accepted client {addr}");

                spawner.spawn_local(handle(stream, hardware_sender.clone())).unwrap();
            }
            Err(e) => {
                error!("Error: {e}");
            }
        }
    }
}

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
