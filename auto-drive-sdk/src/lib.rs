#![no_std]

pub mod the_beginner;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> () {
    the_beginner::print(1);
    the_beginner::delay(50);
    the_beginner::print(50);
    the_beginner::set_onboard_led_color(255,0,0);
    the_beginner::delay(1000);
    the_beginner::set_onboard_led_color(0,255, 0);
    the_beginner::delay(2000);
    the_beginner::print(9999);
    the_beginner::set_onboard_led_color(255,255, 255);
    the_beginner::delay(1000);
    the_beginner::set_onboard_led_color(0,0, 0);
    the_beginner::delay(2000);
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
