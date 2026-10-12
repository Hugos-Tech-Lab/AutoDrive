use crate::the_beginner_car::TheBeginnerCar;

pub struct PhysicalHardware {
    the_beginner_cars: Vec<TheBeginnerCar>,
}

impl PhysicalHardware {
  pub fn new() {
    //
  }

  pub fn is_connected(id: i32) -> bool {
    false
  }

  pub fn the_beginner_cars(&self) -> &Vec<TheBeginnerCar> {
    &self.the_beginner_cars
  }
}
