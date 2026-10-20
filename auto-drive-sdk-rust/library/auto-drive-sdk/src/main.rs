#![no_main]
#![no_std]

use auto_drive_sdk::{hardware::the_beginner_car::TheBeginnerCar, utils::{delay}};

#[auto_drive_sdk::main]
fn main(the_beginner_car: TheBeginnerCar) {


    loop {
    // Wide left turn
    the_beginner_car.set_motor_left_speed(60);
    the_beginner_car.set_motor_right_speed(100);
    delay(1200);

    // Gentle transition
    the_beginner_car.set_motor_left_speed(75);
    the_beginner_car.set_motor_right_speed(90);
    delay(300);

    // Wide right turn
    the_beginner_car.set_motor_left_speed(100);
    the_beginner_car.set_motor_right_speed(60);
    delay(1200);

    // Gentle transition
    the_beginner_car.set_motor_left_speed(90);
    the_beginner_car.set_motor_right_speed(75);
    delay(300);

    }
}
