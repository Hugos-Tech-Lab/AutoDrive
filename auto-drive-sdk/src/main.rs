#![no_main]
#![no_std]

use auto_drive_sdk::hardware::the_beginner;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> () {
  the_beginner::set_onboard_led_color(10,0,0);
}


#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
