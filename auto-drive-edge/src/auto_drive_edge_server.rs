use crate::{
    devices_config::{DeviceConfig, DeviceMetadata, DeviceType},
    the_beginner_cars::TheBeginnerCars,
};

pub mod http_endpoints;

pub struct AutoDriveEdgeServer {
    the_beginner_cars: TheBeginnerCars,
}

impl AutoDriveEdgeServer {
    pub fn new(config: DeviceConfig) -> Self {
        let the_beginner_cars: Vec<DeviceMetadata> = config
            .devices
            .iter()
            .map(|d| d.clone())
            .filter(|device| device.device_type == DeviceType::TheBeginnerCar)
            .collect();

        let the_beginner_cars = TheBeginnerCars::new(the_beginner_cars);

        Self { the_beginner_cars }
    }

    pub async fn run(self) {
        self.the_beginner_cars.run().await;
    }
}
