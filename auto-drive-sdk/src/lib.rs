#![no_std]

use crate::the_beginner::TheBeginnerAutoDrive;

pub mod the_beginner;

struct BasicAutoDrive;

impl TheBeginnerAutoDrive for BasicAutoDrive {
    fn run() {
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
        // the_beginner::set_onboard_led_color(0,0, 0);
    }
}

register!(BasicAutoDrive);
