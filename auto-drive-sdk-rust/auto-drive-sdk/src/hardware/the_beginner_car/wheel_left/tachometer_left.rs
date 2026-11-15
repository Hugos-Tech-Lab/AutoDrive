use the_beginner_car_wasm_sdk::guest;

pub struct TachometerLeft {
    //
}

impl TachometerLeft {
    pub fn new() -> Self {
        Self {}
    }

    pub fn pulses_per_rotation(&self) -> u32 {
        unsafe { guest::get_wheel_left_pulses_per_rotation() }
    }

    pub fn total_pulses(&self) -> i32 {
        unsafe { guest::get_wheel_left_total_pulses() }
    }
}
