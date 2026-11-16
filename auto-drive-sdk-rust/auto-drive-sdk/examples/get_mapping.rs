#![no_main]
#![no_std]

use auto_drive_sdk::{
    hardware::the_beginner_car::TheBeginnerCar,
    utils::{delay, print},
};
use the_beginner_car_wasm_sdk::guest::{get_wheel_left_pulses_per_rotation, set_motor_left_pwm, set_motor_left_rpm};


pub fn get_rpm(the_beginner_car: TheBeginnerCar) -> (f32, f32) {
    let (left_pulses, right_pulses) = get_pulses_in_time(the_beginner_car, 1000);
    unsafe{
        let pulses_per_rotation_left = get_wheel_left_pulses_per_rotation();
        let pulses_per_rotation_right = get_wheel_right_pulses_per_rotation();
    }
    let left_max_rpm = (left_pulses as f32 / pulses_per_rotation_left) * 60.0;
    let right_max_rpm = (right_pulses as f32 / pulses_per_rotation_right) * 60.0;
    (left_max_rpm, right_max_rpm)
}

pub fn get_pulses_in_time(the_beginner_car: TheBeginnerCar, duration_ms: u64) -> (i32, i32) {
    let start_pulses_left = the_beginner_car.wheel_left.tachometer.total_pulses();
    let start_pulses_right = the_beginner_car.wheel_right.tachometer.total_pulses();
    delay(duration_ms);
    let end_pulses_left = the_beginner_car.wheel_left.tachometer.total_pulses();
    let end_pulses_right = the_beginner_car.wheel_right.tachometer.total_pulses();
    let left_pulses = end_pulses_left - start_pulses_left;
    let right_pulses = end_pulses_right - start_pulses_right;
    (left_pulses, right_pulses)
}

#[auto_drive_sdk::main]
fn main(the_beginner_car: TheBeginnerCar) {
    print("Starting mapping procedure");

    let maximum_pwm: i8 = 127;
    let mut minimum_pwm: i8 = 0;

    // Step 1: Determine maximum RPM of the motor
    the_beginner_car.wheel_left.motor.set_pwm(maximum_pwm);
    the_beginner_car.wheel_right.motor.set_pwm(maximum_pwm);
    delay(500);

    let (left_max_rpm, right_max_rpm) = get_rpm(the_beginner_car);
    
    print(&format!(
        "Max RPM: Left: {}, Right: {}",
        left_max_rpm, right_max_rpm
    ));

    // Step 2: Determine minimum RPM of the motor (stall threshold)
    for pwm in 0..=maximum_pwm {
        the_beginner_car.wheel_left.motor.set_pwm(pwm);
        the_beginner_car.wheel_right.motor.set_pwm(pwm);
        
        let (pulses_left, pulses_right) = get_pulses_in_time(the_beginner_car, 500);
        if pulses_left > 5 && pulses_right > 5 {
            minimum_pwm = pwm;
            break;
        }
    }

    // Step 3: Get mapping of PWM to RPM
    let mut mapping_left: Vec<(i8, f32)> = Vec::new();
    let mut mapping_right: Vec<(i8, f32)> = Vec::new();
    for pwm in minimum_pwm..=maximum_pwm {
        the_beginner_car.wheel_left.motor.set_pwm(pwm);
        the_beginner_car.wheel_right.motor.set_pwm(pwm);
        let (left_rpm, right_rpm) = get_rpm(the_beginner_car);
        mapping_left.push((pwm, left_rpm));
        mapping_right.push((pwm, right_rpm));
    }
    
    print("Left Wheel:");
    for (pwm, rpm) in mapping_left {
        print(&format!("PWM: {}, RPM: {}", pwm, rpm));
    }
    print("Right Wheel:");
    for (pwm, rpm) in mapping_right {
        print(&format!("PWM: {}, RPM: {}", pwm, rpm));
    }
}
