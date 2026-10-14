//! A physical small robot that's driven around the Tech Lab.
//! Use this crate to read its input / control it.
mod ffi {
    #[link(wasm_import_module = "host")]
    unsafe extern "C" {
        /// takes u8 so we don't wait too long (i. 255ms max) between cancellation checks
        pub fn set_onboard_led_color(r: u8, g: u8, b: u8);
        pub fn set_motor_a_speed(speed: i8);
        pub fn set_motor_b_speed(speed: i8);
        pub fn get_ir_utmost_front_utmost_left_value() -> u16;
        pub fn get_ir_utmost_front_utmost_right_value() -> u16;
    }
}

pub struct TheBeginnerCar;

impl TheBeginnerCar {
    fn new() -> Self {
        Self {}
    }

    pub fn set_onboard_led_color(r: u8, g: u8, b: u8) {
        unsafe {
            ffi::set_onboard_led_color(r, g, b);
        }
    }

    /// Sets the motor speed
    pub fn set_motor_a_speed(&self, speed: i8) {
        unsafe { ffi::set_motor_a_speed(speed) }
    }

    pub fn set_motor_b_speed(&self, speed: i8) {
        unsafe { ffi::set_motor_b_speed(speed) }
    }

    pub fn get_ir_utmost_front_utmost_left_value(&self) -> u16 {
        unsafe { ffi::get_ir_utmost_front_utmost_left_value() }
    }

    pub fn get_ir_utmost_front_utmost_right_value(&self) -> u16 {
        unsafe { ffi::get_ir_utmost_front_utmost_right_value() }
    }
}

#[doc(hidden)]
pub fn __auto_drive_create_beginner_car() -> TheBeginnerCar {
    TheBeginnerCar::new()
}

