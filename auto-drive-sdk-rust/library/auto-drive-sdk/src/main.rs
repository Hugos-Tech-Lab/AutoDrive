#![no_main]
#![no_std]

use auto_drive_sdk::{hardware::the_beginner_car::TheBeginnerCar, utils::delay};

pub fn is_far_left_on_line(the_beginner_car: &TheBeginnerCar) -> bool {
    the_beginner_car.get_ir_front_far_left_value() > 25_000
}

pub fn is_left_on_line(the_beginner_car: &TheBeginnerCar) -> bool {
    the_beginner_car.get_ir_front_left_value() > 25_000
}

pub fn is_center_on_line(the_beginner_car: &TheBeginnerCar) -> bool {
    the_beginner_car.get_ir_front_center_value() > 25_000
}

pub fn is_right_on_line(the_beginner_car: &TheBeginnerCar) -> bool {
    the_beginner_car.get_ir_front_right_value() > 25_000
}

pub fn is_far_right_on_line(the_beginner_car: &TheBeginnerCar) -> bool {
    the_beginner_car.get_ir_front_far_right_value() > 25_000
}

pub fn is_back_center_on_line(the_beginner_car: &TheBeginnerCar) -> bool {
    the_beginner_car.get_ir_back_center_value() > 25_000
}

pub fn is_back_left_on_line(the_beginner_car: &TheBeginnerCar) -> bool {
    the_beginner_car.get_ir_back_left_value() > 25_000
}

pub fn are_all_front_on_line(the_beginner_car: &TheBeginnerCar) -> bool {
    is_far_left_on_line(&the_beginner_car)
        && is_left_on_line(&the_beginner_car)
        && is_center_on_line(&the_beginner_car)
        && is_right_on_line(&the_beginner_car)
        && is_far_right_on_line(&the_beginner_car)
}

pub fn is_everything_on_line(the_beginner_car: &TheBeginnerCar) -> bool {
    are_all_front_on_line(&the_beginner_car)
        && is_back_center_on_line(&the_beginner_car)
        && is_back_left_on_line(&the_beginner_car)
}

#[auto_drive_sdk::main]
fn main(the_beginner_car: TheBeginnerCar) {
        the_beginner_car.set_motor_left_speed(50);
        the_beginner_car.set_motor_right_speed(50);

        delay(10000);

    // while is_everything_on_line(&the_beginner_car) {
    //     the_beginner_car.set_motor_left_speed(100);
    //     the_beginner_car.set_motor_right_speed(100);
    // }
    // the_beginner_car.set_motor_left_speed(50);
    // the_beginner_car.set_motor_right_speed(50);
    // delay(500);
    // loop {
    //     the_beginner_car.set_motor_left_speed(20);
    //     the_beginner_car.set_motor_right_speed(20);

    //     if is_far_left_on_line(&the_beginner_car) {
    //         the_beginner_car.set_motor_left_speed(-40);
    //         the_beginner_car.set_motor_right_speed(60);
    //     } else if is_left_on_line(&the_beginner_car) {
    //         the_beginner_car.set_motor_left_speed(40);
    //         the_beginner_car.set_motor_right_speed(60);
    //     } else if is_center_on_line(&the_beginner_car) {
    //         the_beginner_car.set_motor_left_speed(60);
    //         the_beginner_car.set_motor_right_speed(60);
    //     } else if is_right_on_line(&the_beginner_car) {
    //         the_beginner_car.set_motor_left_speed(60);
    //         the_beginner_car.set_motor_right_speed(40);
    //     } else if is_far_right_on_line(&the_beginner_car) {
    //         the_beginner_car.set_motor_left_speed(60);
    //         the_beginner_car.set_motor_right_speed(-40);
    //     }

    //     if are_all_front_on_line(&the_beginner_car) {
    //         the_beginner_car.set_motor_left_speed(100);
    //         the_beginner_car.set_motor_right_speed(100);
    //     }

    //     if is_everything_on_line(&the_beginner_car) {
    //         break;
    //     }

    //     delay(10);
    // }
}
