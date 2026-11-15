use the_beginner_car_wasm_sdk::guest;

use crate::hardware::the_beginner_car::{
    ir_sensor_array::IrSensorArray,
    wheel_left::{__auto_drive_create_wheel_left, WheelLeft},
};
pub mod ir_sensor_array;
pub mod wheel_left;

pub struct TheBeginnerCar {
    pub wheel_left: WheelLeft,
    pub ir_sensor_array: IrSensorArray,
}

impl TheBeginnerCar {
    fn new() -> Self {
        Self {
            wheel_left: __auto_drive_create_wheel_left(),
        }
    }

    pub fn turn_left(&self) {
        // self.wheel_left.
    }

    pub fn turn_right(&self) {
        //
    }

    pub fn set_speed(left_amount: i8, right_amount: i8) {
        //
    }

    pub fn set_onboard_led_color(r: u8, g: u8, b: u8) {
        unsafe {
            guest::set_onboard_led_color(r, g, b);
        }
    }

    /// Sets the motor speed
    pub fn set_motor_left_speed(&self, speed: i8) {
        unsafe { guest::set_motor_left_speed(speed) }
    }

    pub fn set_motor_right_speed(&self, speed: i8) {
        unsafe { guest::set_motor_right_speed(speed) }
    }

    pub fn set_led_left_on(&self) {
        unsafe { guest::set_led_left_on() }
    }

    pub fn set_led_left_off(&self) {
        unsafe { guest::set_led_left_off() }
    }

    pub fn set_led_right_on(&self) {
        unsafe { guest::set_led_right_on() }
    }

    pub fn set_led_right_off(&self) {
        unsafe { guest::set_led_right_off() }
    }

    pub fn get_ir_back_center_value(&self) -> u16 {
        unsafe { guest::get_ir_back_center_value() }
    }

    pub fn get_ir_front_far_left_value(&self) -> u16 {
        unsafe { guest::get_ir_front_far_left_value() }
    }

    pub fn get_ir_front_left_value(&self) -> u16 {
        unsafe { guest::get_ir_front_left_value() }
    }

    pub fn get_ir_front_center_value(&self) -> u16 {
        unsafe { guest::get_ir_front_center_value() }
    }

    pub fn get_ir_front_right_value(&self) -> u16 {
        unsafe { guest::get_ir_front_right_value() }
    }

    pub fn get_ir_front_far_right_value(&self) -> u16 {
        unsafe { guest::get_ir_front_far_right_value() }
    }

    pub fn get_ir_back_left_value(&self) -> u16 {
        unsafe { guest::get_ir_back_left_value() }
    }
}

#[doc(hidden)]
pub fn __auto_drive_create_beginner_car() -> TheBeginnerCar {
    TheBeginnerCar::new()
}
