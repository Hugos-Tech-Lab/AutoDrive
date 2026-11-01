use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering},
    },
    time::{Duration, Instant},
};

use esp_idf_svc::hal::{
    delay::FreeRtos,
    gpio::{Input, InterruptType, PinDriver},
};

use crate::{
    auto_script::cancellation_token::CancellationToken,
    hardware::{led::Led, motor::Motor},
};

pub mod led;
pub mod motor;
pub mod wheel;

#[derive(Debug, PartialEq)]
pub enum RequestOrigin {
    AutoScript(i32),
    Manual,
}

pub struct TheBeginnerCarHardware {
    // auto_script: AutoScript,
    allowed_controller: RequestOrigin,
    wheel_left: Motor,
    wheel_right: Motor,
    wheel_left_hall_sensor: PinDriver<'static, Input>,
    // led_left: Led,
    led_right: Led,
    battery_voltage: f32,
    // front
    ir_front_far_left_latest_reading: Arc<AtomicU16>,
    ir_front_left_latest_reading: Arc<AtomicU16>,
    ir_front_center_latest_reading: Arc<AtomicU16>,
    ir_front_right_latest_reading: Arc<AtomicU16>,
    ir_front_far_right_latest_reading: Arc<AtomicU16>,
    // back
    ir_back_center_latest_reading: Arc<AtomicU16>,
    ir_back_left_latest_reading: Arc<AtomicU16>,
}

impl TheBeginnerCarHardware {
    pub fn new(
        wheel_left: Motor,
        wheel_right: Motor,
        // led_left: Led,
        led_right: Led,
        // front
        ir_front_far_left_latest_reading: Arc<AtomicU16>,
        ir_front_left_latest_reading: Arc<AtomicU16>,
        ir_front_center_latest_reading: Arc<AtomicU16>,
        ir_front_right_latest_reading: Arc<AtomicU16>,
        ir_front_far_right_latest_reading: Arc<AtomicU16>,
        // back
        ir_back_center_latest_reading: Arc<AtomicU16>,
        ir_back_left_latest_reading: Arc<AtomicU16>,
    ) -> Self {
        Self {
            allowed_controller: RequestOrigin::Manual,
            wheel_left,
            wheel_right,
            // led_left,
            led_right,
            battery_voltage: 0.0,
            ir_front_far_left_latest_reading,
            ir_front_left_latest_reading,
            ir_front_center_latest_reading,
            ir_front_right_latest_reading,
            ir_front_far_right_latest_reading,
            ir_back_center_latest_reading,
            ir_back_left_latest_reading,
            wheel_left_hall_sensor: todo!(),
        }
    }

    pub fn led_left(&mut self) -> &mut Led {
        &mut self.led_right // TODO fix this
    }

    pub fn led_right(&mut self) -> &mut Led {
        &mut self.led_right
    }

    pub fn wheel_left(&mut self) -> &mut Motor {
        &mut self.wheel_left
    }

    pub fn wheel_right(&mut self) -> &mut Motor {
        &mut self.wheel_left
    }

    // pub fn move_forward_by_cm(&mut self, cm: i8) {
    // self.wheel_left.
    // }

    pub fn wheel_left_wait_for_rotations(
        &mut self,
        target_rotations: u32,
        timeout: u32,
        ct: CancellationToken,
    ) -> u32 {
        //
        target_duration
    }

    pub fn wheel_right_wait_for_rotations(&self, target_rotations: u32, timeout: u32) -> u32 {
        0
    }

    pub fn set_wheel_left_speed(&mut self, speed: i8, _origin: RequestOrigin) {
        // if self.allowed_controller != origin {
        // panic!(); // TODO: not authorized
        // }

        self.wheel_left.set_speed(speed).unwrap();
    }

    pub fn set_wheel_right_speed(&mut self, speed: i8, _origin: RequestOrigin) {
        // if self.allowed_controller != origin {
        //     panic!(); // TODO: not allowed
        // }
        self.wheel_right.set_speed(speed).unwrap();
    }

    pub fn set_allowed_controller(&mut self, controller: RequestOrigin) {
        self.allowed_controller = controller;
    }

    pub fn notify_battery_reading(&mut self, battery_voltage: f32) {
        self.battery_voltage = battery_voltage;
    }

    pub fn get_ir_front_far_left_value(&self) -> u16 {
        self.ir_front_far_left_latest_reading
            .load(Ordering::Relaxed)
    }

    pub fn get_ir_front_left_value(&self) -> u16 {
        self.ir_front_left_latest_reading.load(Ordering::Relaxed)
    }

    pub fn get_ir_front_center_value(&self) -> u16 {
        self.ir_front_center_latest_reading.load(Ordering::Relaxed)
    }

    pub fn get_ir_front_right_value(&self) -> u16 {
        self.ir_front_right_latest_reading.load(Ordering::Relaxed)
    }

    pub fn get_ir_front_far_right_value(&self) -> u16 {
        self.ir_front_far_right_latest_reading
            .load(Ordering::Relaxed)
    }

    pub fn get_ir_back_center_value(&self) -> u16 {
        self.ir_back_center_latest_reading.load(Ordering::Relaxed)
    }

    pub fn get_ir_back_left_value(&self) -> u16 {
        self.ir_back_left_latest_reading.load(Ordering::Relaxed)
    }

    pub fn stop_all(&mut self) {
        self.wheel_left.stop().unwrap();
        self.wheel_right.stop().unwrap();
        // self.led_left.stop().unwrap();
        self.led_right.stop().unwrap();
    }

    pub fn run_calibration(&mut self) {
        // self.
    }
}
