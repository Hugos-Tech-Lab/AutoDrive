#![no_std]

use crate::the_beginner::TheBeginnerAutoDrive;

pub mod the_beginner;

struct BasicAutoDrive;

impl TheBeginnerAutoDrive for BasicAutoDrive {
    fn run() {
        the_beginner::delay(50);
        the_beginner::print(3);
        the_beginner::delay(100);
        the_beginner::print(1);
        the_beginner::delay(200);
        the_beginner::print(0);
        the_beginner::delay(1000);
        the_beginner::print(2);
        the_beginner::delay(2000);
        the_beginner::print(4);
        // the_beginner::print(45);
        // the_beginner::delay(50);
        // the_beginner::print(45);
    }
}

register!(BasicAutoDrive);
