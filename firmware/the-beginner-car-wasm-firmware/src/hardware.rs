use crate::hardware::motor::Motor;

pub mod motor;

#[derive(Debug, PartialEq)]
pub enum RequestOrigin {
    AutoScript(i32),
    Manual,
}

pub struct TheBeginnerCarHardware {
    // auto_script: AutoScript,
    allowed_controller: RequestOrigin,
    wheel_a: Motor,
    wheel_b: Motor,
}

impl TheBeginnerCarHardware {
    pub fn new(wheel_a: Motor, wheel_b: Motor) -> Self {
        Self {
            allowed_controller: RequestOrigin::Manual,
            wheel_a,
            wheel_b,
        }
    }

    pub fn set_wheel_a_speed(&mut self, speed: i8, origin: RequestOrigin) {
        if self.allowed_controller != origin {
            panic!(); // TODO: not authorized
        }
        self.wheel_a.set_speed(speed).unwrap();
    }

    pub fn set_wheel_b_speed(&mut self, speed: i8, origin: RequestOrigin) {
        if self.allowed_controller != origin {
            panic!(); // TODO: not allowed
        }
        self.wheel_b.set_speed(speed).unwrap();
    }

    pub fn set_allowed_controller(&mut self, controller: RequestOrigin) {
        self.allowed_controller = controller;
    }

    pub fn stop(&mut self) {
        //
    }

    // pub fn switch_
}
