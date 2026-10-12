use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{the_beginner_car_comms::TheBeginnerCarComms, the_beginner_car_driver::TheBeginnerCarDriver};



pub struct PhysicalHardware {
    the_beginner_cars: Vec<TheBeginnerCarComms>,
    the_beginner_car_drivers: Vec<Arc<Mutex<TheBeginnerCarDriver>>>,
}

impl PhysicalHardware {
  pub fn new() {
    //
  }

  pub fn is_connected(id: i32) -> bool {
    false
  }

  pub fn the_beginner_cars(&self) -> &Vec<TheBeginnerCarComms> {
    &self.the_beginner_cars
  }
}
