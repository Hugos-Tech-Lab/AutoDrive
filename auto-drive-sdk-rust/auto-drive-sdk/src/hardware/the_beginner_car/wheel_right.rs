use the_beginner_car_wasm_sdk::guest;

use crate::hardware::the_beginner_car::{
    wheel_left::{motor_left::MotorLeft, tachometer_left::TachometerLeft},
    wheel_right::{motor_right::MotorRight, tachometer_right::TachometerRight},
};

pub mod motor_right;
pub mod tachometer_right;

pub struct WheelRight {
    pub motor: MotorRight,
    pub tachometer: TachometerRight,
}

impl WheelRight {
    pub fn new() -> Self {
        Self {
            motor: MotorRight::new(),
            tachometer: TachometerRight::new(),
        }
    }

    pub fn circumference_cm(&self) -> f32 {
        unsafe { guest::get_wheel_right_circumference_cm() }
    }
}

#[doc(hidden)]
pub fn __auto_drive_create_wheel_right() -> WheelRight {
    WheelRight::new()
}
