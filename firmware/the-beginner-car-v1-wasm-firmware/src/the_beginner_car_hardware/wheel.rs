use crate::the_beginner_car_hardware::wheel::{motor::Motor, tachometer::Tachometer};

pub mod motor;
pub mod tachometer;

pub struct Wheel {
    motor: Motor,
    tachometer: Tachometer,
}

impl Wheel {
    pub fn new(motor: Motor, tachometer: Tachometer) -> Self {
        Self { motor, tachometer }
    }

    pub fn motor(&self) -> &Motor {
        &self.motor
    }

    pub fn tachometer(&self) -> &Tachometer {
        &self.tachometer
    }
}
