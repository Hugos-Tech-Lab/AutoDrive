use std::{
    error::Error,
    sync::{Arc, atomic::AtomicU32},
    time::Duration,
};
use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_otlp::{MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{metrics::{SdkMeterProvider, exporter::PushMetricExporter}, trace::SpanExporter as _};
use tokio::{net::TcpStream, time::timeout};

use crate::{auto_drive_edge_server::AutoDriveEdgeServer, devices_config::DeviceConfig};

pub mod auto_drive_edge_server;
pub mod devices_config;
pub mod the_beginner_car;
pub mod the_beginner_cars;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let contents = include_str!("../device_config.toml");
    let device_config: DeviceConfig = toml::from_str(&contents)?;
    let auto_drive_edge_server = AutoDriveEdgeServer::new(device_config);

    // 1. Build the OTLP gRPC Exporter using the modern builder pattern
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


    auto_drive_edge_server.run().await;
       

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

    // 5. Explicitly flush and shutdown before application exit
    span_exporter.shutdown().unwrap();
    meter_provider.shutdown().unwrap();
    Ok(())
}
