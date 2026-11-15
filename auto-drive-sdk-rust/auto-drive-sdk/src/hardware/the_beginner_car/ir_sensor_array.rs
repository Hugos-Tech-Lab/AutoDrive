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
}
