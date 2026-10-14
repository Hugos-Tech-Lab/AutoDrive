#![no_main]
#![no_std]

use auto_drive_sdk::hardware::the_beginner::{self, get_ir_sensor_a_value, get_ir_sensor_b_value};

#[unsafe(no_mangle)]
pub extern "C" fn main() -> () {
let threshold_off: i32 = 1000;

let base_speed: i8 = 70;
let turn_speed: i8 = 30;

loop {
    let sensor_a = get_ir_sensor_a_value() as i32;
    let sensor_b = get_ir_sensor_b_value() as i32;

    let a_on_line = sensor_a > threshold_off;
    let b_on_line = sensor_b > threshold_off;

    if !a_on_line && !b_on_line {
        // Both sensors see white: move forward.
        the_beginner::set_motor_a_speed(base_speed);
        the_beginner::set_motor_b_speed(base_speed);

    } else if a_on_line && !b_on_line {
        // Left sensor sees black: steer left.
        the_beginner::set_motor_a_speed(turn_speed);
        the_beginner::set_motor_b_speed(base_speed);

    } else if !a_on_line && b_on_line {
        // Right sensor sees black: steer right.
        the_beginner::set_motor_a_speed(base_speed);
        the_beginner::set_motor_b_speed(turn_speed);
    } else {
        the_beginner::set_motor_a_speed(0);
        the_beginner::set_motor_b_speed(0);
    }

    the_beginner::delay(10);
}

}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
