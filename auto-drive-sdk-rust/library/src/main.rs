#![no_main]
#![no_std]

use auto_drive_sdk::hardware::the_beginner;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> () {
  loop {
  the_beginner::set_onboard_led_color(0,255,0);
  // the_beginner::print("hi");
  the_beginner::delay(500);
  the_beginner::set_onboard_led_color(0,0,255);
  the_beginner::delay(500);

  the_beginner::print("hello");
  the_beginner::set_onboard_led_color(255,255,255);
  the_beginner::delay(500);
    the_beginner::print("hi");

  // the_beginner::set_onboard_led_color(0,0,100);
  // // the_beginner::print("hi");
  // the_beginner::delay(50);
  // the_beginner::set_onboard_led_color(0,0,0);
  // the_beginner::delay(50);
  }



  // loop {
  //   the_beginner::delay(500);
  //   // the_beginner::print("aaaaaaaaaaaaaaaaaaaaaaaa");
  // }
}


#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
