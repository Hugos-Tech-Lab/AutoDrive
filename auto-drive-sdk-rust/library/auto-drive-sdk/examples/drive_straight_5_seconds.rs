#![no_main]
#![no_std]

use auto_drive_sdk::{hardware::the_beginner_car::TheBeginnerCar, utils::{delay, print}};

#[auto_drive_sdk::main]
fn main(the_beginner_car: TheBeginnerCar) {
  the_beginner_car.set_motor_left_speed(-100);
  the_beginner_car.set_motor_right_speed(100);
  print("running");
  delay(2000);
}
