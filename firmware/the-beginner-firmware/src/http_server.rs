use std::sync::{Arc};

use edge_nal::TcpBind;

pub mod auto_script;
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

pub type SmallServer = Server<2, 1024, 16>;

#[derive(Serialize)]
struct FirmwareUpdate200Response {
    status: String,
}

pub struct HttpHandler {
    pub auto_script: Arc<AutoScript>,
}

impl HttpHandler {
    pub fn new(auto_script: Arc<AutoScript>) -> Self {
        Self { auto_script }
    }
}

impl Handler for HttpHandler {
    type Error<E>
        = Error<E>
    where
        E: Debug;

    async fn handle<T, const N: usize>(
        &self,
        _task_id: impl Display + Copy,
        conn: &mut Connection<'_, T, N>,
    ) -> Result<(), Self::Error<T::Error>>
    where
        T: Read + Write,
    {
        let request_headers = conn.headers()?;
        let path = request_headers.path;
        let method = request_headers.method;

        match (method, path) {
            (Method::Options, "/autoscript/upload") | (Method::Options, "/autoscript/run") => {
                conn.initiate_response(204, None, &CORS_HEADERS).await?;
            }
            (Method::Post, "/autoscript/upload") => {
                auto_script::upload(conn, self.auto_script.clone()).await?
            }
            (Method::Post, "/autoscript/run") => {
                auto_script::run(conn, self.auto_script.clone()).await?
            }
            (Method::Post, "/autoscript/cancel") => {
                auto_script::cancel(conn, self.auto_script.clone()).await?
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

pub async fn run(server: &mut SmallServer, auto_script: Arc<AutoScript>) -> Result<(), anyhow::Error> {
    let addr ="0.0.0.0:80".parse().unwrap();
    log::info!("Running HTTP server on {addr}");

    let acceptor = edge_nal_std::Stack::new()
        .bind(addr)
        .await?;

    server.run(None, acceptor, HttpHandler { auto_script }).await?;

    Ok(())
}


static CORS_HEADERS: [(&str, &str); 3] = [
    ("Access-Control-Allow-Origin", "*"),
    ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
    ("Access-Control-Allow-Headers", "Content-Type"),
];
