#![no_main]
#![no_std]

use auto_drive_sdk::hardware::the_beginner;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> () {
    loop {
      the_beginner::set_motor_a_speed(0);
      the_beginner::set_motor_b_speed(0);
      the_beginner::delay(1000);
      the_beginner::set_motor_a_speed(0);
      the_beginner::set_motor_b_speed(0);
      the_beginner::delay(1000);

    }

}


#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
