use bytes::Bytes;
use opentelemetry_otlp::{MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{metrics::SdkMeterProvider, trace::SpanExporter as _};
use std::error::Error;
use warp::Filter;

use crate::{auto_drive_edge_server::AutoDriveEdgeServer, devices_config::DeviceConfig};

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
            .and(warp::header::optional::<String>("user-agent"))
            .and(warp::body::bytes())
            .and_then(move |agent: Option<String>, body: Bytes| {
                let driver = driver.clone();

                async move {
                    let data = body;

                    driver.run_auto_script(data.to_vec()).await;

                    Ok::<_, warp::Rejection>(format!(
                        "Hello, whose agent is {}",
                        agent.unwrap_or_else(|| "unknown".to_string())
                    ))
                }
            });

        warp::serve(hi).run(([127, 0, 0, 1], 3030)).await;
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
