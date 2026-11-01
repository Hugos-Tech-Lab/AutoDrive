use crate::hardware::motor::Motor;

pub mod tachometer;

pub struct Wheel {
    motor: Motor,
}
