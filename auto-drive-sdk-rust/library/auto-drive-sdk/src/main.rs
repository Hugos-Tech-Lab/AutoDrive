#![no_main]
#![no_std]

use auto_drive_sdk::{hardware::the_beginner_car::TheBeginnerCar, utils::{delay, print}};

#[auto_drive_sdk::main]
fn main(the_beginner_car: TheBeginnerCar) {
    loop {
        delay(1000);
        print("hi");
    }
}
