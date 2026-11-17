use the_beginner_car_wasm_sdk::guest::{self};

use crate::{
    hardware::the_beginner_car::{wheel_left::WheelLeft, wheel_right::WheelRight},
    utils::delay,
};
pub mod ir_sensor_array;
pub mod wheel_left;
pub mod wheel_right;

pub struct TheBeginnerCar {
    pub wheel_left: WheelLeft,
    pub wheel_right: WheelRight,
    // pub ir_sensor_array: IrSensorArray,
}

impl TheBeginnerCar {
    fn new() -> Self {
        Self {
            wheel_left: WheelLeft::new(),
            wheel_right: WheelRight::new(),
        }
    }

    pub fn rotate_by_radians(&self, radians: f32) {
        if radians == 0.0 {
            return;
        }

        let track_width_cm = 16.4; // TODO: expose this in the car 

        let wheel_left_circumference = self.wheel_left.circumference_cm();
        let wheel_right_circumference = self.wheel_right.circumference_cm();

        let wheel_left_pulses_per_rotation =
            self.wheel_left.tachometer.pulses_per_rotation() as f32;
        let wheel_right_pulses_per_rotation =
            self.wheel_right.tachometer.pulses_per_rotation() as f32;

        // Calculate the arc length each wheel needs to travel: s = (track_width / 2) * |radians|
        let arc_length_cm = (track_width_cm / 2.0) * radians.abs();

        // Convert arc length to target encoder pulses for each wheel
        let target_pulses_left = ((arc_length_cm / wheel_left_circumference)
            * wheel_left_pulses_per_rotation
            + 0.5) as u32;
        let target_pulses_right = ((arc_length_cm / wheel_right_circumference)
            * wheel_right_pulses_per_rotation
            + 0.5) as u32;

        let wheel_left_total_pulses_start = self.wheel_left.tachometer.total_pulses();
        let wheel_right_total_pulses_start = self.wheel_right.tachometer.total_pulses();

        let direction = if radians > 0.0 { 1 } else { -1 };

        self.wheel_left.motor.set_pwm(-70 * direction); // TODO: replace with set_rpm
        self.wheel_right.motor.set_pwm(70 * direction);

        loop {
            let left_elapsed = self
                .wheel_left
                .tachometer
                .total_pulses()
                .abs_diff(wheel_left_total_pulses_start);
            let right_elapsed = self
                .wheel_right
                .tachometer
                .total_pulses()
                .abs_diff(wheel_right_total_pulses_start);

            if left_elapsed >= target_pulses_left {
                self.wheel_left.motor.set_pwm(0);
            }
            if right_elapsed >= target_pulses_right {
                self.wheel_right.motor.set_pwm(0);
            }

            if left_elapsed >= target_pulses_left && right_elapsed >= target_pulses_right {
                break;
            }

            delay(1);
        }
    }

    // Positive angle = counter-clockwise (Left)
    // Negative angle = clockwise (Right)
    pub fn rotate_by_degrees(&self, degrees: f32) {
        let radians = degrees * (core::f32::consts::PI / 180.0);
        self.rotate_by_radians(radians);
    }

    pub fn rotate_left(&self) {
        self.rotate_by_degrees(90.0);
    }

    pub fn rotate_right(&self) {
        self.rotate_by_degrees(-90.0);
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
    // pub fn set_motor_left_speed(&self, speed: i8) {
    //     unsafe { guest::set_motor_left_speed(speed) }
    // }

    // pub fn set_motor_right_speed(&self, speed: i8) {
    //     unsafe { guest::set_motor_right_speed(speed) }
    // }

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
}

#[doc(hidden)]
pub fn __auto_drive_create_beginner_car() -> TheBeginnerCar {
    TheBeginnerCar::new()
}
