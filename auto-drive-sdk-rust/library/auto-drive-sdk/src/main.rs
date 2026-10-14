#![no_main]
#![no_std]

use auto_drive_sdk::{hardware::the_beginner_car::TheBeginnerCar, utils::{delay, print}};

#[auto_drive_sdk::main]
fn main(the_beginner_car: TheBeginnerCar) {
    loop {
        // the_beginner_car.set_motor_a_speed(2);
        delay(1000);
        print("hi");
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
