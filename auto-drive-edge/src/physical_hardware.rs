use std::sync::Arc;

use crate::{
    devices_config::DeviceConfig,
    the_beginner_car_comms::TheBeginnerCarComms,
    auto_script_manager::AutoScriptManager,
};

pub struct PhysicalHardware {
    the_beginner_cars: Vec<TheBeginnerCarComms>,
    pub the_beginner_car_drivers: Vec<Arc<AutoScriptManager>>,
}

impl PhysicalHardware {
    pub fn new(config: DeviceConfig) -> Self {
        let the_beginner_cars: Vec<TheBeginnerCarComms> = config
            .the_beginner_cars
            .iter()
            .map(|d| TheBeginnerCarComms::new(d.clone()))
            .collect();

        let the_beginner_car_drivers = the_beginner_cars
            .iter()
            .map(|the_beginner_car_comms| {
                Arc::new(AutoScriptManager::new(
                    the_beginner_car_comms.outgoing_tx.clone(),
                    the_beginner_car_comms.incoming_tx.clone(),
                ))
            })
            .collect();

        Self {
            the_beginner_cars,
            the_beginner_car_drivers,
        }
    }

    pub fn is_connected(id: i32) -> bool {
        false
    }

    pub fn the_beginner_cars(&self) -> &Vec<TheBeginnerCarComms> {
        &self.the_beginner_cars
    }
}
