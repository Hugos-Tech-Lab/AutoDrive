use crate::{devices_config::{DeviceConfig, DeviceMetadata, DeviceType}, the_beginner_car::TheBeginnerCar};

pub struct AutoDriveEdgeServer {
  the_beginners_cars: Vec<TheBeginnerCar>
}

impl AutoDriveEdgeServer {
  pub fn new (config: DeviceConfig) -> Self {
    let the_beginner_cars: Vec<&DeviceMetadata> = config
        .devices
        .iter()
        .filter(|device| device.device_type == DeviceType::TheBeginnerCar)
        .collect();

      Self { the_beginners_cars: the_beginner_cars.into_iter().map(|device_metadata| TheBeginnerCar::new(device_metadata.clone())).collect() }
    }

    pub async fn run(&self) {
      //
    }
}
