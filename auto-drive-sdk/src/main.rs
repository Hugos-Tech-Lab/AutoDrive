#![no_main]
#![no_std]

use auto_drive_sdk::hardware::the_beginner;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> () {
  the_beginner::set_onboard_led_color(4,0,0);
  the_beginner::print(1);
  the_beginner::print(2);
  the_beginner::print(3);
  the_beginner::print(4);
  the_beginner::print(5);
  the_beginner::print(6);
}


#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
