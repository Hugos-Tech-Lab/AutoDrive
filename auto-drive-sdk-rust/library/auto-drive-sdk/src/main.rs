#![no_main]
#![no_std]

use auto_drive_sdk::{hardware::the_beginner_car::TheBeginnerCar, utils::{delay, print}};

#[auto_drive_sdk::main]
fn main(the_beginner_car: TheBeginnerCar) {
    the_beginner_car.set_motor_a_speed(127);
    the_beginner_car.set_motor_b_speed(127);
    delay(1000);
    the_beginner_car.set_motor_a_speed(0);
    the_beginner_car.set_motor_b_speed(0);
    loop {
        delay(1000);
        print("hi");
    }
}
