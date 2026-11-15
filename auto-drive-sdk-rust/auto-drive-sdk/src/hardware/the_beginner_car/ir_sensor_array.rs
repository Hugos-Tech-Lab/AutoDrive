use the_beginner_car_wasm_sdk::guest;

pub struct IrSensorArray {
    //
}

impl IrSensorArray {
    // The percentage of IR light reflected
    // 0.0 means dark, i.e. the surface absorbs all light (tape)
    // 1.0 means bright, i.e. the surface reflected all light (tiles)
    pub fn front_far_left_reflectance() -> f32 {
        0.0
    }

    pub fn front_far_right() -> f32 {
        0.0
    }

    pub fn get_ir_back_center_value(&self) -> u16 {
        unsafe { guest::get_ir_back_center_value() }
    }

    pub fn get_ir_front_far_left_value(&self) -> u16 {
        unsafe { guest::get_ir_front_far_left_value() }
    }

    pub fn get_ir_front_left_value(&self) -> u16 {
        unsafe { guest::get_ir_front_left_value() }
    }

    pub fn get_ir_front_center_value(&self) -> u16 {
        unsafe { guest::get_ir_front_center_value() }
    }

    pub fn get_ir_front_right_value(&self) -> u16 {
        unsafe { guest::get_ir_front_right_value() }
    }

    pub fn get_ir_front_far_right_value(&self) -> u16 {
        unsafe { guest::get_ir_front_far_right_value() }
    }

    pub fn get_ir_back_left_value(&self) -> u16 {
        unsafe { guest::get_ir_back_left_value() }
    }
}
