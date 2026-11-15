use the_beginner_car_wasm_sdk::guest;

pub struct TachometerRight {
    //
}

impl TachometerRight {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub fn pulses_per_rotation(&self) -> u32 {
        unsafe { guest::get_wheel_right_pulses_per_rotation() }
    }

    pub fn total_pulses(&self) -> i32 {
        unsafe { guest::get_wheel_right_total_pulses() }
    }
}
