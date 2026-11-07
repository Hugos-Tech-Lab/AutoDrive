use std::sync::{Arc, Mutex, atomic::Ordering};

use esp_idf_svc::hal::delay::FreeRtos;

use crate::{
    auto_script::cancellation_token::CancellationToken,
    the_beginner_car_hardware::{
        ir_sensor_array::IrSensorArray,
        led::Led,
        wheel::{Wheel, tachometer::Tachometer},
    },
};

pub mod ir_sensor_array;
pub mod led;
pub mod wheel;

#[derive(Debug, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,
    pub combined_dist: f32,
}

impl Position {
    pub fn add(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.rotation_radians += other.rotation_radians;
        self.combined_dist += other.combined_dist;
    }
}

// #[derive(Debug, PartialEq)]
// pub enum RequestOrigin {
//     AutoScript(i32),
//     Calibration,
//     Manual,
// }

pub struct KalmanFilter {
    position: Arc<Mutex<Option<Position>>>, // 0.1: rotation, distance moved
                                            // 0.2: rotation, distance moved
}

impl KalmanFilter {
    pub fn new(left_tachometer: Tachometer, right_tachometer: Tachometer) -> Self {
        let position = Arc::new(Mutex::new(Some(Position {
            x: 0.0,
            y: 0.0,
            rotation_radians: 0.0,
            combined_dist: 0.0,
        })));

        std::thread::Builder::new()
            .name("kalman_filter".to_string())
            .spawn({
                let position = position.clone();
                || Self::run(left_tachometer, right_tachometer, position)
            })
            .unwrap();

        Self { position }
    }

    pub fn run(
        left_tachometer: Tachometer,
        right_tachometer: Tachometer,
        position: Arc<Mutex<Option<Position>>>,
    ) {
        // let mut state_state = Vec::new(); // dim n x 1
        // let mut sigma_prev = Vec::new(); // dim n x n
        // let mut measured_pos = Vec::new(); // dim n x n
        // let mut process_noise = Vec::new(); // dim n x n
        // let mut measurement_noise = Vec::new(); // dim n x n
        let dt: f32; // timestep in ms
        let wingspan: f32 = 18.5;
        let mut left_tachometer_total_pulses_previous = left_tachometer.get_total_pulses();
        let mut right_tachometer_total_pulses_previous = right_tachometer.get_total_pulses();
        let mut current_rotation = 0.0; // TODO: derived from current position

        loop {
            let left_tachometer_total_pulses = left_tachometer.get_total_pulses();
            let right_tachometer_total_pulses = right_tachometer.get_total_pulses();
            let left_tachometer_pulses =
                left_tachometer_total_pulses - left_tachometer_total_pulses_previous;
            let right_tachometer_pulses =
                right_tachometer_total_pulses - right_tachometer_total_pulses_previous;

            left_tachometer_total_pulses_previous = left_tachometer_total_pulses;
            right_tachometer_total_pulses_previous = right_tachometer_total_pulses;

            let distance_left_cm = left_tachometer_pulses as f32 * left_tachometer.cm_per_pulse();
            let distance_right_cm =
                right_tachometer_pulses as f32 * right_tachometer.cm_per_pulse();
            let position_delta = Self::predict(
                distance_left_cm,
                distance_right_cm,
                wingspan,
                current_rotation,
            );

            current_rotation += position_delta.rotation_radians;

            {
                let mut position_guard = position.lock().unwrap();
                if let Some(pos) = position_guard.as_mut() {
                    // Replace `&other_position` with whatever value you actually need to add
                    pos.add(position_delta);
                }
            }

            FreeRtos::delay_ms(10);
        }
    }

    pub fn update(&mut self, position: Position) {
        let mut position_guard = self.position.lock().unwrap();
        position_guard.replace(position);
    }

    pub fn predict(
        distance_left_cm: f32,
        distance_right_cm: f32,
        wingspan: f32,
        current_rotation: f32,
    ) -> Position {
        let distance = (distance_left_cm + distance_right_cm) / 2.0;
        let delta_theta = (distance_right_cm - distance_left_cm) / wingspan;

        let (dx, dy) = if delta_theta.abs() < 1e-6 {
            (
                distance * current_rotation.cos(),
                distance * current_rotation.sin(),
            )
        } else {
            let radius = distance / delta_theta;

            (
                radius * ((current_rotation + delta_theta).sin() - current_rotation.sin()),
                -radius * ((current_rotation + delta_theta).cos() - current_rotation.cos()),
            )
        };

        Position {
            x: dx,
            y: dy,
            rotation_radians: delta_theta,
            combined_dist: distance,
        }
    }

    // expected to be called, i.e. every 100ms
    pub fn position(&mut self) -> Option<Position> {
        let position = self.position.lock().unwrap();
        position.clone()
    }
}

// Checks the position and stops if out of bounds
pub struct EmergencyStop {
    //
}

pub struct TheBeginnerCarHardware {
    ir_sensor_array: Arc<IrSensorArray>,
    wheel_left: Wheel,
    wheel_right: Wheel,
    kalman_filter: KalmanFilter,
    emergency_stop: EmergencyStop,
    // led_left: Led,
    // led_right: Led,
    battery_voltage: f32,
}

impl TheBeginnerCarHardware {
    pub fn new(
        wheel_left: Wheel,
        wheel_right: Wheel,
        // led_left: Led,
        // led_right: Led,
        ir_sensor_array: Arc<IrSensorArray>,
    ) -> Self {
        let kalman_filter = KalmanFilter::new(
            wheel_left.tachometer().clone(),
            wheel_right.tachometer().clone(),
        );

        Self {
            wheel_left,
            wheel_right,
            // led_left,
            // led_right,
            battery_voltage: 0.0,
            ir_sensor_array,
            kalman_filter,
            emergency_stop: EmergencyStop {},
        }
    }

    // pub fn led_left(&mut self) -> &mut Led {
    //     &mut self.led_right // TODO fix this
    // }

    // pub fn led_right(&mut self) -> &mut Led {
    //     &mut self.led_right
    // }

    pub fn wheel_left(&mut self) -> &mut Wheel {
        &mut self.wheel_left
    }

    pub fn wheel_right(&mut self) -> &mut Wheel {
        &mut self.wheel_right
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
        0
        // target_duration
    }

    pub fn wheel_right_wait_for_rotations(&self, target_rotations: u32, timeout: u32) -> u32 {
        0
    }

    pub fn set_wheel_left_speed(&mut self, speed: i8) {
        // if self.allowed_controller != origin {
        // panic!(); // TODO: not authorized
        // }

        // self.wheel_left.set_speed(speed).unwrap();
    }

    pub fn set_wheel_right_speed(&mut self, speed: i8) {
        // if self.allowed_controller != origin {
        //     panic!(); // TODO: not allowed
        // }
        // self.wheel_right.set_speed(speed).unwrap();
    }

    pub fn set_allowed_controller(&mut self) {
        // self.allowed_controller = controller;
    }

    pub fn notify_position_from_camera(&mut self, position: Position) {
        self.kalman_filter.update(position);
    }

    pub fn notify_battery_reading(&mut self, battery_voltage: f32) {
        self.battery_voltage = battery_voltage;
    }

    pub fn get_ir_front_far_left_value(&self) -> u16 {
        // self.ir_front_far_left_latest_reading
        //     .load(Ordering::Relaxed)
        0
    }

    pub fn get_ir_front_left_value(&self) -> u16 {
        // self.ir_front_left_latest_reading.load(Ordering::Relaxed)
        0
    }

    pub fn get_ir_front_center_value(&self) -> u16 {
        // self.ir_front_center_latest_reading.load(Ordering::Relaxed)
        0
    }

    pub fn get_ir_front_right_value(&self) -> u16 {
        // self.ir_front_right_latest_reading.load(Ordering::Relaxed)
        0
    }

    pub fn get_ir_front_far_right_value(&self) -> u16 {
        // self.ir_front_far_right_latest_reading
        //     .load(Ordering::Relaxed)
        0
    }

    pub fn get_ir_back_center_value(&self) -> u16 {
        // self.ir_back_center_latest_reading.load(Ordering::Relaxed)
        0
    }

    pub fn get_ir_back_left_value(&self) -> u16 {
        // self.ir_back_left_latest_reading.load(Ordering::Relaxed)
        0
    }

    pub fn stop_all(&mut self) {
        self.wheel_left.motor().stop().unwrap();
        self.wheel_right.motor().stop().unwrap();
        // self.led_left.stop().unwrap();
        // self.led_right.stop().unwrap();
    }

    pub fn run_calibration(&mut self) {
        // self.
    }
}
