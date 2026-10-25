#![no_main]
#![no_std]

use auto_drive_sdk::{hardware::the_beginner_car::TheBeginnerCar, utils::{delay, print}};

#[auto_drive_sdk::main]
fn main(the_beginner_car: TheBeginnerCar) {
    print("Starting infinite figure 8 loop");

    loop {
        the_beginner_car.set_motor_left_speed(-50);
        the_beginner_car.set_motor_right_speed(50);
        delay(3000);
    }
}