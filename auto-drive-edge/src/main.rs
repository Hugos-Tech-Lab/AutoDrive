use crate::{
    auto_drive_edge_server::AutoDriveEdgeServer,
    auto_drive_edge_service::v1::auto_drive_edge_service_server::AutoDriveEdgeServiceServer,
    devices_config::DeviceConfig, grpc_service::GrpcService,
};

use env_logger::Env;
use log::info;
use opentelemetry_otlp::{MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{metrics::SdkMeterProvider, trace::SpanExporter as _};
use std::error::Error;

pub mod auto_drive_edge_server;
pub mod auto_script_manager;
pub mod devices_config;
pub mod grpc_service;
pub mod the_beginner_car_comms;
pub mod vehicle_hardware_telemetry;

pub mod auto_drive_edge_service {
    pub mod v1 {
        tonic::include_proto!("auto_drive_edge_service.v1");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_source_path(true) // Enables file and line number
        .init();

    #[cfg(feature = "development")]
    info!("auto-drive-cloud launched in development mode");

    #[cfg(feature = "production")]
    {
        const GIT_HASH: &str = env!("GIT_HASH");
        info!("Starting auto-drive-edge in production mode at git hash ({GIT_HASH})");
    }

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
    let auto_drive_edge_server = AutoDriveEdgeServer::new(device_config);

    let grpc_service = GrpcService::new(
        auto_drive_edge_server.auto_script_manager,
        auto_drive_edge_server.the_beginner_cars.clone(),
    );

    tokio::spawn(async move {
        let address = "0.0.0.0:50051".parse().unwrap();

        info!("serving gRPC to {address}");
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
