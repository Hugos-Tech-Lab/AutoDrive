use crate::{
    auto_drive_edge_server::AutoDriveEdgeServer,
    auto_drive_edge_service::v1::auto_drive_edge_service_server::AutoDriveEdgeServiceServer,
    devices_config::DeviceConfig, grpc_service::GrpcService,
};
use opentelemetry_otlp::{MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{metrics::SdkMeterProvider, trace::SpanExporter as _};
use std::error::Error;

pub mod auto_drive_edge_server;
pub mod devices_config;
pub mod grpc_service;
pub mod local_http;
pub mod physical_hardware;
pub mod the_beginner_car_comms;
pub mod the_beginner_car_auto_script;
pub mod vehicle_hardware_telemetry;

pub mod auto_drive_edge_service {
    pub mod v1 {
        tonic::include_proto!("auto_drive_edge_service.v1");
    }
}

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

    let grpc_service = GrpcService::new(driver.clone());

    tokio::spawn(async move {
        let address = "127.0.0.1:50051".parse().unwrap();

        println!("serving gRPC to {address}");
        tonic::transport::Server::builder()
            .add_service(AutoDriveEdgeServiceServer::new(grpc_service))
            .serve(address)
            .await
            .expect("gRPC server failed");
    })
    .await
    .unwrap();

    // 5. Explicitly flush and shutdown before application exit
    span_exporter.shutdown().unwrap();
    meter_provider.shutdown().unwrap();
    Ok(())
}
