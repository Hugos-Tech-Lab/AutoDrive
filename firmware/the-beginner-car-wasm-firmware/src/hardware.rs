use std::sync::{Arc, atomic::{AtomicU16, Ordering}};

use crate::hardware::motor::Motor;

pub mod motor;

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
    motor_battery: f32,
    mcu_battery: f32,
    ir_sensor_a: Arc<AtomicU16>,
    ir_sensor_b: Arc<AtomicU16>,
}

impl TheBeginnerCarHardware {
    pub fn new(wheel_a: Motor, wheel_b: Motor, ir_sensor_a: Arc<AtomicU16>, ir_sensor_b: Arc<AtomicU16>) -> Self {
        Self {
            allowed_controller: RequestOrigin::Manual,
            wheel_a,
            wheel_b,
            motor_battery: 0.0,
            mcu_battery: 0.0,
            ir_sensor_a,
            ir_sensor_b
        }
    }

    pub fn set_wheel_a_speed(&mut self, speed: i8, origin: RequestOrigin) {
        // if self.allowed_controller != origin {
        // panic!(); // TODO: not authorized
        // }

        self.wheel_a.set_speed(speed).unwrap();
    }

    pub fn set_wheel_b_speed(&mut self, speed: i8, origin: RequestOrigin) {
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

    pub fn get_ir_sensor_a_value(&self) -> u16 {
        self.ir_sensor_a.load(Ordering::Relaxed)
    }

    pub fn get_ir_sensor_b_value(&self) -> u16 {
        self.ir_sensor_b.load(Ordering::Relaxed)
    }

    pub fn stop(&mut self) {
        //
    }

    // pub fn switch_
}
