use core::fmt::{Debug, Display};
use std::{
    sync::{Arc, mpsc},
    thread,
};

use edge_http::io::server::{Connection, Handler};
use edge_http::io::Error;
use edge_http::Method;
use embedded_io_async::{Read, Write};
use log::info;
use serde::Serialize;

use crate::autoscript::{AutoScript, AutoScriptRunProgress};

#[derive(Serialize)]
struct FirmwareUpdate200Response {
    status: String,
}

// 1. Add your state to the HttpHandler struct
pub struct HttpHandler {
    pub auto_script: Arc<AutoScript>,
}

impl HttpHandler {
    pub fn new(auto_script: Arc<AutoScript>) -> Self {
        Self { auto_script }
    }
}

impl Handler for HttpHandler {
    type Error<E> = Error<E> where E: Debug;

    async fn handle<T, const N: usize>(
        &self,
        _task_id: impl Display + Copy,
        conn: &mut Connection<'_, T, N>,
    ) -> Result<(), Self::Error<T::Error>>
    where
        T: Read + Write,
    {
        let headers = conn.headers()?;
        let path = headers.path;
        let method = headers.method;
        info!("{:?}", path);
        info!("{:?}", method);


        let cors_headers = [
            ("Access-Control-Allow-Origin", "*"),
            ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
            ("Access-Control-Allow-Headers", "Content-Type"),
        ];

        match (method, path) {
            // --- OPTIONS Handlers ---
            (Method::Options, "/autoscript/upload") | (Method::Options, "/autoscript/run") => {
                conn.initiate_response(204, None, &cors_headers).await?;
            }

            // // --- POST Upload Handler ---
            (Method::Post, "/autoscript/upload") => {
                info!("processing upload");

                // edge-http usually parses content_len for us
                let content_length = headers.headers.content_len();
                log::info!("Starting autoscript update, content length: {:?}", content_length);

                let capacity = content_length.map(|c| c as usize).unwrap_or(2048);
                let mut data = Vec::with_capacity(capacity);
                let mut buf = [0u8; 2048];
                let mut total = 0usize;
                // TODO: add progress to response
                loop {
                    let read = conn.read(&mut buf).await?;
                    if read == 0 {
                        break;
                    }
                    data.extend_from_slice(&buf[..read]);
                    total += read;
                    log::debug!("Autoscript: received {} bytes", total);
                    
                    // Break early if we've reached the expected content length to avoid hanging 
                    // on keep-alive connections
                    if let Some(cl) = content_length {
                        if total >= cl as usize {
                            break;
                        }
                    }
                }

                log::info!("Autoscript download complete: {} bytes", total);

                let res = self.auto_script.install(data).await;

                if let Ok(res) = res {
                    log::error!("{:?}", res);
                }

                // if let Err(e) = res {
                //     log::error!("Failed to install script: {:?}", e);
                //     conn.initiate_response(500, Some("Internal Server Error"), &cors_headers).await?;
                //     return Ok(());
                // }

                let body = serde_json::to_vec(&FirmwareUpdate200Response {
                    status: "ok".to_string(),
                }).unwrap();

                let mut res_headers = cors_headers.to_vec();
                res_headers.push(("Content-Type", "application/json"));
                
                conn.initiate_response(200, Some("OK"), &res_headers).await?;
                conn.write_all(&body).await?;
            }

            // --- POST Run Handler ---
            (Method::Post, "/autoscript/run") => {
                log::info!("Starting autoscript execution stream...");
                let (tx, rx) = mpsc::sync_channel::<AutoScriptRunProgress>(10);

                let auto_script_clone = self.auto_script.clone();
                let res = auto_script_clone.run(tx).await.unwrap();
                info!("{:?}", res);


                let mut res_headers = cors_headers.to_vec();
                res_headers.push(("Content-Type", "application/x-ndjson"));
                res_headers.push(("Transfer-Encoding", "chunked"));

                conn.initiate_response(200, Some("OK"), &res_headers).await?;

                // Note: Using try_recv + thread::yield_now() prevents this loop from completely 
                // blocking the async executor while waiting for the thread to produce logs. 
                // If you use embassy or tokio, consider substituting thread::yield_now() with their async yield/sleep.
                loop {
                    match rx.try_recv() {
                        Ok(chunk) => {
                            let mut bytes = serde_json::to_vec(&chunk).unwrap();
                            bytes.push(b'\n'); // CRITICAL for NDJSON framing
                            conn.write_all(&bytes).await?;
                        }
                        Err(mpsc::TryRecvError::Empty) => {
                            // Yield back to the executor
                            thread::yield_now(); 
                        }
                        Err(mpsc::TryRecvError::Disconnected) => {
                            break;
                        }
                    }
                }

                log::info!("Autoscript execution stream ended.");
            }

            // // --- POST Cancel Handler ---
            // (Method::Post, "/autoscript/cancel") => {
            //     log::info!("Starting cancel stream...");
                
            //     self.auto_script.cancel();
                
            //     log::info!("done stream...");

            //     let body = serde_json::to_vec(&FirmwareUpdate200Response {
            //         status: "ok".to_string(),
            //     }).unwrap();

            //     let mut res_headers = cors_headers.to_vec();
            //     res_headers.push(("Content-Type", "application/json"));
                
            //     conn.initiate_response(200, Some("OK"), &res_headers).await?;
            //     conn.write_all(&body).await?;
            // }

            // --- Fallbacks (Not Found / Wrong Method) ---
            (_, "/autoscript/upload") | (_, "/autoscript/run") | (_, "/autoscript/cancel") => {
                conn.initiate_response(405, Some("Method Not Allowed"), &[]).await?;
            }
            _ => {
                conn.initiate_response(404, Some("Not Found"), &[]).await?;
            }
        }

        Ok(())
    }
}
