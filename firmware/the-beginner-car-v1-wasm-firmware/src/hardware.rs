use std::sync::{
    Arc,
    atomic::{AtomicU16, Ordering},
};

use crate::hardware::{led::Led, motor::Motor};

pub mod motor;
pub mod led;

#[derive(Debug, PartialEq)]
pub enum RequestOrigin {
    AutoScript(i32),
    Manual,
}

pub struct TheBeginnerCarHardware {
    // auto_script: AutoScript,
    allowed_controller: RequestOrigin,
    wheel_a: Motor,
    wheel_b: Motor,
    led_left: Led,
    led_right: Led,
    motor_battery: f32,
    mcu_battery: f32,
    ir_utmost_front_utmost_left_latest_reading: Arc<AtomicU16>,
    ir_utmost_front_utmost_right_latest_reading: Arc<AtomicU16>,
}

impl TheBeginnerCarHardware {
    pub fn new(
        wheel_a: Motor,
        wheel_b: Motor,
        led_left: Led,
        led_right: Led,
        ir_utmost_front_utmost_left_latest_reading: Arc<AtomicU16>,
        ir_utmost_front_utmost_right_latest_reading: Arc<AtomicU16>,
    ) -> Self {
        Self {
            allowed_controller: RequestOrigin::Manual,
            wheel_a,
            wheel_b,
            led_left,
            led_right,
            motor_battery: 0.0,
            mcu_battery: 0.0,
            ir_utmost_front_utmost_left_latest_reading,
            ir_utmost_front_utmost_right_latest_reading,
        }
    }

    pub fn led_left(&mut self) -> &mut Led {
        &mut self.led_left
    }

    pub fn led_right(&mut self) -> &mut Led {
        &mut self.led_right
    }

    pub fn set_wheel_a_speed(&mut self, speed: i8, _origin: RequestOrigin) {
        // if self.allowed_controller != origin {
        // panic!(); // TODO: not authorized
        // }

        self.wheel_a.set_speed(speed).unwrap();
    }

    pub fn set_wheel_b_speed(&mut self, speed: i8, _origin: RequestOrigin) {
        // if self.allowed_controller != origin {
        //     panic!(); // TODO: not allowed
        // }
        self.wheel_b.set_speed(speed).unwrap();
    }

    pub fn set_allowed_controller(&mut self, controller: RequestOrigin) {
        self.allowed_controller = controller;
    }

    pub fn notify_battery_readings(&mut self, motor_battery: f32, mcu_battery: f32) {
        self.motor_battery = motor_battery;
        self.mcu_battery = mcu_battery;
    }

    pub fn get_ir_utmost_front_utmost_left_value(&self) -> u16 {
        self.ir_utmost_front_utmost_left_latest_reading
            .load(Ordering::Relaxed)
    }

    pub fn get_ir_utmost_front_utmost_right_value(&self) -> u16 {
        self.ir_utmost_front_utmost_right_latest_reading
            .load(Ordering::Relaxed)
    }

    pub fn stop_all(&mut self) {
        self.wheel_a.stop().unwrap();
        self.wheel_b.stop().unwrap();
        self.led_left.stop().unwrap();
        self.led_right.stop().unwrap();
    }

    // pub fn switch_
}
