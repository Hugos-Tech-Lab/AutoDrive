#![no_main]
#![no_std]

use auto_drive_sdk::{hardware::the_beginner_car::TheBeginnerCar, utils::delay};

pub fn is_left_on_line(the_beginner_car: &TheBeginnerCar) -> bool {
    the_beginner_car.get_ir_front_left_value() > 25_000
}

pub fn is_center_on_line(the_beginner_car: &TheBeginnerCar) -> bool {
    the_beginner_car.get_ir_front_center_value() > 25_000
}

pub fn is_right_on_line(the_beginner_car: &TheBeginnerCar) -> bool {
    the_beginner_car.get_ir_front_right_value() > 25_000
}

pub fn is_back_on_line(the_beginner_car: &TheBeginnerCar) -> bool {
    the_beginner_car.get_ir_back_center_value() > 25_000
}

#[auto_drive_sdk::main]
fn main(the_beginner_car: TheBeginnerCar) {
    while is_left_on_line(&the_beginner_car)
        && is_center_on_line(&the_beginner_car)
        && is_right_on_line(&the_beginner_car)
        && is_back_on_line(&the_beginner_car)
    {
        the_beginner_car.set_motor_left_speed(127);
        // the_beginner_car.set_motor_right_speed(127);
    }

    loop {
        if is_left_on_line(&the_beginner_car)
            && is_center_on_line(&the_beginner_car)
            && is_right_on_line(&the_beginner_car)
            && is_back_on_line(&the_beginner_car)
        {
            the_beginner_car.set_motor_left_speed(0);
            the_beginner_car.set_motor_right_speed(0);
            continue;
        }

        if is_center_on_line(&the_beginner_car) {
            the_beginner_car.set_motor_left_speed(50);
            the_beginner_car.set_motor_right_speed(50);
        } else {
            the_beginner_car.set_motor_left_speed(40);
            the_beginner_car.set_motor_right_speed(40);
        }

        if is_left_on_line(&the_beginner_car) {
            the_beginner_car.set_motor_left_speed(-20);
            the_beginner_car.set_motor_right_speed(50);
        } else if is_right_on_line(&the_beginner_car) {
            the_beginner_car.set_motor_left_speed(50);
            the_beginner_car.set_motor_right_speed(-20);
        } else if !is_center_on_line(&the_beginner_car) {
            the_beginner_car.set_motor_left_speed(60);
            the_beginner_car.set_motor_right_speed(-20);
        }

        delay(10);
    }
}
