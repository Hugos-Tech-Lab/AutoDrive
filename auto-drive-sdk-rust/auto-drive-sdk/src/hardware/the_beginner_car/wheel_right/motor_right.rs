use the_beginner_car_wasm_sdk::guest;

pub struct MotorRight {
    //
}

impl MotorRight {
    pub(crate) fn new() -> Self {
        Self {}
    }

    // pub fn set_rpm(&self, rpm: i32) {
    //     unsafe { guest::set_motor_right_rpm(rpm) }
    // }

    pub fn set_pwm(&self, pwm: i8) {
        unsafe { guest::set_motor_right_pwm(pwm) }
    }
}
