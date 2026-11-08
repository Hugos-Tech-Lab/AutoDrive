use std::sync::{Arc, Mutex, atomic::Ordering};

use esp_idf_svc::hal::delay::FreeRtos;
use the_beginner_car_spi_protocol::Position;

use crate::{
    auto_script::cancellation_token::CancellationToken,
    the_beginner_car_hardware::{
        ir_sensor_array::IrSensorArray,
        wheel::{Wheel, tachometer::Tachometer},
    },
};

pub mod ir_sensor_array;
pub mod led;
pub mod wheel;

// #[derive(Debug, PartialEq)]
// pub enum RequestOrigin {
//     AutoScript(i32),
//     Calibration,
//     Manual,
// }

pub struct KalmanFilterIterateState {
    pub dt: f32,                                     // timestep in ms
    pub wingspan: f32,                               // = 18.5;
    pub left_tachometer_total_pulses_previous: u32,  //,  = self.left_tachometer.get_total_pulses();
    pub right_tachometer_total_pulses_previous: u32, // = self.right_tachometer.get_total_pulses();
    pub current_rotation: f32, //  = 0.0; // TODO: derived from current position
}

#[derive(Debug, Clone)]
pub struct KalmanFilter {
    position: Arc<Mutex<Option<Position>>>, // 0.1: rotation, distance moved
    // 0.2: rotation, distance moved
    left_tachometer: Tachometer,
    right_tachometer: Tachometer,
}

impl KalmanFilter {
    pub fn new(left_tachometer: Tachometer, right_tachometer: Tachometer) -> Self {
        let position = Arc::new(Mutex::new(Some(Position {
            x: 0.0,
            y: 0.0,
            rotation_radians: 0.0,
            timestamp_unix_epoch_ms: 0,
        })));

        Self {
            position,
            left_tachometer,
            right_tachometer,
        }
    }

    pub fn create_iterate_state(&self) -> KalmanFilterIterateState {
        KalmanFilterIterateState {
            dt: 0.0,
            wingspan: 18.5,
            left_tachometer_total_pulses_previous: self.left_tachometer.get_total_pulses(),
            right_tachometer_total_pulses_previous: self.right_tachometer.get_total_pulses(),
            current_rotation: 0.0,
        }
    }

    pub fn iterate(&self, state: &mut KalmanFilterIterateState) -> Option<Position> {
        let left_tachometer_total_pulses = self.left_tachometer.get_total_pulses();
        let right_tachometer_total_pulses = self.right_tachometer.get_total_pulses();
        let left_tachometer_pulses =
            left_tachometer_total_pulses - state.left_tachometer_total_pulses_previous;
        let right_tachometer_pulses =
            right_tachometer_total_pulses - state.right_tachometer_total_pulses_previous;

        state.left_tachometer_total_pulses_previous = left_tachometer_total_pulses;
        state.right_tachometer_total_pulses_previous = right_tachometer_total_pulses;

        let distance_left_cm = left_tachometer_pulses as f32 * self.left_tachometer.cm_per_pulse();
        let distance_right_cm =
            right_tachometer_pulses as f32 * self.right_tachometer.cm_per_pulse();
        let position_delta = Self::predict(
            distance_left_cm,
            distance_right_cm,
            state.wingspan,
            state.current_rotation,
        );

        state.current_rotation += position_delta.rotation_radians;

        let position;
        {
            let mut position_guard = self.position.lock().unwrap();
            if let Some(pos) = position_guard.as_mut() {
                pos.add(position_delta);
                position = Some(pos.clone());
            } else {
                position = None;
            }
        }

        position
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
            timestamp_unix_epoch_ms: 0,
        }
    }

    // expected to be called, i.e. every 100ms
    pub fn position(&self) -> Option<Position> {
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
        kalman_filter: KalmanFilter,
    ) -> Self {
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
        self.kalman_filter.update(Position {
            x: position.x,
            y: position.y,
            rotation_radians: position.rotation_radians,
            timestamp_unix_epoch_ms: 0,
        });
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

    pub fn get_position(&self) -> Option<Position> {
        self.kalman_filter.position()
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
