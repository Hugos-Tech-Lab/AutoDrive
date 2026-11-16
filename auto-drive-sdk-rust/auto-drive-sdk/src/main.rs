#![no_main]
#![no_std]

// use alloc::{format, string::ToString, vec};
use auto_drive_sdk::{
    hardware::the_beginner_car::TheBeginnerCar,
    utils::{delay, print},
};

// extern crate alloc;

// use talc::{source::Claim, *};

// #[global_allocator]
// static TALC: TalcLock<spinning_top::RawSpinlock, Claim> = TalcLock::new(unsafe {
//     static mut INITIAL_HEAP: [u8; min_first_heap_size::<DefaultBinning>() + 100000] =
//         [0; min_first_heap_size::<DefaultBinning>() + 100000];

//     Claim::array(&raw mut INITIAL_HEAP)
// });

#[auto_drive_sdk::main]
fn main(the_beginner_car: TheBeginnerCar) {
    // let hi = vec![0, 1, 2, 3];
    // panic!();
    // delay(1000);

    // dbg!();
    print(format_args!("hi: {}", 2).as_str().unwrap());
    delay(1000);

    // print(&format!("hi").to_string());
    // the_beginner_car.wheel_left.motor.set_pwm(125);
    // the_beginner_car.wheel_right.motor.set_pwm(15);
    // delay(1000);
    // the_beginner_car.rotate_left();
    // the_beginner_car.rotate_left();
    // the_beginner_car.rotate_left();
    // the_beginner_car.rotate_left();
    // the_beginner_car.set_motor_right_speed(50);

    // delay(10000);

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
