use std::sync::Arc;

use edge_nal::TcpBind;

pub mod auto_script;
pub mod hardware;
pub mod info;
pub mod logs;
pub mod update;
pub mod verify_and_set_valid;

use core::fmt::{Debug, Display};

use edge_http::Method;
use edge_http::io::Error;
use edge_http::io::server::{Connection, Handler, Server};
use embedded_io_async::{Read, Write};
use serde::Serialize;

use crate::auto_script::AutoScript;

pub type SmallServer = Server<2, 1024, 8>;

#[derive(Serialize)]
struct FirmwareUpdate200Response {
    status: String,
}

pub struct HttpHandler {
}

impl HttpHandler {
    pub fn new() -> Self {
        Self { }
    }
}

impl Handler for HttpHandler {
    type Error<E>
        = Error<E>
    where
        E: Debug;

    async fn handle<T, const N: usize>(
        &self,
        task_id: impl Display + Copy,
        conn: &mut Connection<'_, T, N>,
    ) -> Result<(), Self::Error<T::Error>>
    where
        T: Read + Write,
    {
        let request_headers = conn.headers()?;
        let path = request_headers.path;
        let method = request_headers.method;

        log::info!("HTTP request: task={} {:?} {}", task_id, method, path);

        match (method, path) {
            (Method::Options, "/autoscript/upload")
            | (Method::Options, "/autoscript/run")
            | (Method::Options, "/hardware/set_motor_a_speed")
            | (Method::Options, "/hardware/set_motor_b_speed")
            | (Method::Options, "/logs") => {
                conn.initiate_response(204, None, &CORS_HEADERS).await?;
            }
            (Method::Post, "/hardware/set_motor_a_speed") => {
                log::info!("waiting");
                hardware::set_motor_a_speed(conn).await?;
                log::info!("response");
            }
            (Method::Get, "/logs") => logs::logs(conn).await?,
            (Method::Get, "/coredump") => logs::coredump(conn).await?,
            (Method::Post, "/hardware/set_motor_b_speed") => {
                hardware::set_motor_b_speed(conn).await?
            }
            (_, "/autoscript/upload") | (_, "/autoscript/run") | (_, "/autoscript/cancel") => {
                conn.initiate_response(405, Some("Method Not Allowed"), &[])
                    .await?;
            }
            _ => {
                conn.initiate_response(404, Some("Not Found"), &[]).await?;
            }
        }

        Ok(())
    }
}

pub async fn run(
    server: &mut SmallServer
) -> Result<(), anyhow::Error> {
    let addr = "0.0.0.0:80".parse().unwrap();
    log::info!("Running HTTP server on {addr}");

    let acceptor = edge_nal_std::Stack::new().bind(addr).await?;
    // let acceptor = EspTcpAcceptor::new(2);

    server
        .run(None, acceptor, HttpHandler { })
        .await?;

    Ok(())
}

static CORS_HEADERS: [(&str, &str); 3] = [
    ("Access-Control-Allow-Origin", "*"),
    ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
    ("Access-Control-Allow-Headers", "Content-Type"),
];
