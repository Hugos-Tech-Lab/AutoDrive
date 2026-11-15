use crate::hardware::the_beginner_car::wheel_left::{
    motor_left::MotorLeft, tachometer_left::TachometerLeft,
};

pub mod motor_left;
pub mod tachometer_left;

pub struct WheelLeft {
    pub motor: MotorLeft,
    pub tachometer: TachometerLeft,
}

impl WheelLeft {
    pub fn new() -> Self {
        Self {
            motor: todo!(),
            tachometer: todo!(),
        }
    }
}

#[doc(hidden)]
pub fn __auto_drive_create_wheel_left() -> WheelLeft {
    WheelLeft::new()
}
