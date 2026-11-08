use std::sync::Arc;

use the_beginner_car_spi_protocol::Position;

use crate::{
    auto_script::cancellation_token::CancellationToken,
    hardware_config::HardwareConfig,
    the_beginner_car_hardware::{
        ir_sensor_array::IrSensorArray, kalman_filter::KalmanFilter, wheel::Wheel,
    },
};

pub mod ir_sensor_array;
pub mod kalman_filter;
pub mod led;
pub mod wheel;

// #[derive(Debug, PartialEq)]
// pub enum RequestOrigin {
//     AutoScript(i32),
//     Calibration,
//     Manual,
// }

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
    hardware_config: HardwareConfig,
}

impl TheBeginnerCarHardware {
    pub fn new(
        wheel_left: Wheel,
        wheel_right: Wheel,
        // led_left: Led,
        // led_right: Led,
        ir_sensor_array: Arc<IrSensorArray>,
        kalman_filter: KalmanFilter,
        hardware_config: HardwareConfig,
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
            hardware_config,
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
