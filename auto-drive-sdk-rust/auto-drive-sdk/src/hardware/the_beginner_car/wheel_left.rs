use the_beginner_car_wasm_sdk::guest;

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
    pub(crate) fn new() -> Self {
        Self {
            motor: MotorLeft::new(),
            tachometer: TachometerLeft::new(),
        }
    }

    pub fn circumference_cm(&self) -> f32 {
        unsafe { guest::get_wheel_right_circumference_cm() }
    }
}
