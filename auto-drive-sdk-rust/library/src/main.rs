#![no_main]
#![no_std]

use auto_drive_sdk::hardware::the_beginner::{self, get_ir_sensor_a_value, get_ir_sensor_b_value};

#[unsafe(no_mangle)]
pub extern "C" fn main() -> () {
    let mut last_direction = 1;

    loop {
        let sensor_a = get_ir_sensor_a_value(); // Left sensor
        let sensor_b = get_ir_sensor_b_value(); // Right sensor

        if sensor_a > 1400 && sensor_b > 1400 {
            // Both sensors detect the line: drive straight
            the_beginner::set_motor_a_speed(80);
            the_beginner::set_motor_b_speed(80);
        } else if sensor_a > 1400 {
            // Line is on the left: turn left
            the_beginner::set_motor_a_speed(30);
            the_beginner::set_motor_b_speed(80);
        } else if sensor_b > 1400 {
            // Line is on the right: turn right
            the_beginner::set_motor_a_speed(80);
            the_beginner::set_motor_b_speed(30);
        } else {
            // Neither sensor detects the line: search for it
            the_beginner::set_motor_a_speed(50);
            the_beginner::set_motor_b_speed(-50);
        }

        the_beginner::delay(10);
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
