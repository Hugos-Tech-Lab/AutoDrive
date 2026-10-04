use crate::hardware::motor::Motor;

pub mod motor;

pub struct TheBeginnerCarHardware<'d> {
    wheel_a: Motor<'d>,
    wheel_b: Motor<'d>,
}

impl<'d> TheBeginnerCarHardware<'d> {
    pub fn new(wheel_a: Motor<'d>, wheel_b: Motor<'d>) -> Self {
        Self { wheel_a, wheel_b }
    }
}

