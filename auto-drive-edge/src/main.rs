use crate::{auto_drive_edge_server::AutoDriveEdgeServer, devices_config::DeviceConfig};
use bytes::Bytes;
use futures_util::StreamExt;
use opentelemetry_otlp::{MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{metrics::SdkMeterProvider, trace::SpanExporter as _};
use std::{convert::Infallible, error::Error, future};
use tokio_stream::wrappers::BroadcastStream;
use warp::Filter;

pub mod auto_drive_edge_server;
pub mod devices_config;
pub mod local_http;
pub mod physical_hardware;
pub mod the_beginner_car_comms;
pub mod the_beginner_car_driver;
pub mod vehicle_hardware_telemetry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        std::env::set_var("OTEL_METRIC_EXPORT_INTERVAL", "5000");
    }

    let span_exporter = SpanExporter::builder()
        .with_tonic() // Instructs it to use gRPC via tonic
        .with_endpoint("http://localhost:4317")
        .build()?;

    let metric_exporter = MetricExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()?;

    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(metric_exporter)
        .build();

    opentelemetry::global::set_meter_provider(meter_provider.clone());

    let contents = include_str!("../device_config.toml");
    let device_config: DeviceConfig = toml::from_str(&contents)?;
    let _auto_drive_edge_server = AutoDriveEdgeServer::new(device_config);

    let driver = _auto_drive_edge_server
        .hardware
        .the_beginner_car_drivers
        .first()
        .unwrap()
        .clone();

    let server = tokio::spawn(async move {
        let driver = driver; // assuming driver is available

        let hi = warp::path!("autoscript" / "run")
            .and(warp::post())
            .and(warp::body::bytes())
            .and_then({
                let driver = driver.clone();
                move |body: Bytes| {
                    let driver = driver.clone();
                async move {
                    let data = body;
                    let broadcast_rx = driver.run_auto_script(data.to_vec()).await;
                    let rx_stream = BroadcastStream::new(broadcast_rx);
                    let event_stream = rx_stream.filter_map(|result| {
                        let output = match result {
                            Ok(msg) => {
                                match msg {
                                        the_beginner_car_tcp_protocol::TheBeginnerCarOutgoingMessages::Log { log } => {
                                            // Send this message
                                            Some(Ok::<_, Infallible>(
                                                warp::sse::Event::default().data(log.message)
                                            ))
                                        }
                                        _ => {
                                            // Skip this message
                                            None
                                        }
                                    }
                            }
                            Err(e) => {
                                // Forward stream errors
                                Some(Ok::<_, Infallible>(
                                    warp::sse::Event::default()
                                        .event("error")
                                        .data(e.to_string()),
                                ))
                            }
                        };

                        // Wrap the Option into a Future to satisfy filter_map's trait bounds
                        future::ready(output)
                    });

                    // 3. Reply with the SSE stream. (Optional: add keep_alive pings)
                    Ok::<_, warp::Rejection>(warp::sse::reply(
                        warp::sse::keep_alive().stream(event_stream),
                    ))
                }
            }});

        let cancel = warp::path!("autoscript" / "cancel")
            .and(warp::post())
            .and(warp::body::bytes())
            .and_then(move |body: Bytes| {
                let driver = driver.clone();

                async move {
                    let data = body;

                    driver.run_auto_script(data.to_vec()).await;

                    Ok::<_, warp::Rejection>(format!("Cancelled"))
                }
            });

        warp::serve(hi.or(cancel)).run(([127, 0, 0, 1], 3030)).await;
    });

    // auto_drive_edge_server.run().await;

    // let counter = meter
    //     .u64_counter("battery.connections")
    //     .build();

    // let attrs: [KeyValue; 1] = [KeyValue::new("build", "build_a")];

    // counter.add(1, &[]);

    // 4. Create your tracer and start recording spans
    // let tracer = global::tracer("rust-grpc-service");

    // tracer.in_span("main-operation", |cx| {
    //     let span = cx.span();
    //     span.add_event("Doing some heavy work...", vec![]);
    // });

    server.await.unwrap();

    // 5. Explicitly flush and shutdown before application exit
    span_exporter.shutdown().unwrap();
    meter_provider.shutdown().unwrap();
    Ok(())
}
