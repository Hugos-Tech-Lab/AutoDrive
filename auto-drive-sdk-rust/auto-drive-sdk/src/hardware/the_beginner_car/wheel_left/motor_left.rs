use the_beginner_car_wasm_sdk::guest;

pub struct MotorLeft {
    //
}

impl MotorLeft {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub fn set_rpm(&self, rpm: i32) {
        unsafe { guest::set_motor_left_rpm(rpm) }
    }

    pub fn set_pwm(&self, pwm: i8) {
        unsafe { guest::set_motor_left_pwm(pwm) }
    }
}

// #[doc(hidden)]
// pub fn __auto_drive_create_motor_left() -> MotorLeft {
//     MotorLeft::new()
// }
