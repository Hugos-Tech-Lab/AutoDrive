use the_beginner_car_wasm_sdk::guest;

pub struct TheBeginnerCar;

impl TheBeginnerCar {
    fn new() -> Self {
        Self {}
    }

    pub fn set_onboard_led_color(r: u8, g: u8, b: u8) {
        unsafe {
            guest::set_onboard_led_color(r, g, b);
        }
    }

    /// Sets the motor speed
    pub fn set_motor_a_speed(&self, speed: i8) {
        unsafe { guest::set_motor_a_speed(speed) }
    }

    pub fn set_motor_b_speed(&self, speed: i8) {
        unsafe { guest::set_motor_b_speed(speed) }
    }

    pub fn get_ir_utmost_front_utmost_left_value(&self) -> u16 {
        unsafe { guest::get_ir_utmost_front_utmost_left_value() }
    }

    pub fn get_ir_utmost_front_utmost_right_value(&self) -> u16 {
        unsafe { guest::get_ir_utmost_front_utmost_right_value() }
    }
}

#[doc(hidden)]
pub fn __auto_drive_create_beginner_car() -> TheBeginnerCar {
    TheBeginnerCar::new()
}

