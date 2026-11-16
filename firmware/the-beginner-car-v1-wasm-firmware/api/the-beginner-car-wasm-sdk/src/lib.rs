#![no_std]

pub mod macros;

// Invoke the macro to generate the guest module and host trait
define_wasm_api! {
    // ==================
    // ===== UTILS ======
    // ==================
    //
    /// takes u8 so we don't wait too long (i.e. 255ms max) between cancellation checks
    fn delay(milliseconds: u8);

    // fn delay_for_motor_left_rotations(count: u32, timeout: u32) -> u32;

    // fn delay_for_motor_right_rotations(count: u32, timeout: u32) -> u32;

    /// prints from memory
    fn print(ptr: u32, len: u32);

    // ==================
    // === WHEEL LEFT ===
    // ==================
    //
    // This is a % from -100 to 100
    fn set_motor_left_speed(speed: i8);

    fn set_motor_left_pwm(pwm: i8);

    fn set_motor_left_rpm(rpm: i32);

    fn get_wheel_left_circumference_cm() -> f32;

    fn get_wheel_left_pulses_per_rotation() -> u32;

    fn get_wheel_left_total_pulses() -> i32;

    // ===================
    // === WHEEL RIGHT ===
    // ===================
    //
    // This is a % from -100 to 100
    fn set_motor_right_speed(speed: i8);

    // From -128 to 127
    fn set_motor_right_pwm(pwm: i8);

    // From -128 to 127
    fn set_motor_right_rpm(rpm: i32);

    fn get_wheel_right_circumference_cm() -> f32;

    fn get_wheel_right_pulses_per_rotation() -> u32;

    fn get_wheel_right_total_pulses() -> i32;

    /// Sets the onboard LED color
    fn set_onboard_led_color(r: u8, g: u8, b: u8);

    fn set_led_left_on();

    fn set_led_left_off();

    fn set_led_right_on();

    fn set_led_right_off();

    // ===================
    // = IR SENSOR ARRAY =
    // ===================

    fn get_ir_front_far_left_value() -> u16;

    fn get_ir_front_left_value() -> u16;

    fn get_ir_front_center_value() -> u16;

    fn get_ir_front_right_value() -> u16;

    fn get_ir_front_far_right_value() -> u16;

    fn get_ir_back_center_value() -> u16;

    fn get_ir_back_left_value() -> u16;

    // ==================
    // ==== POSITION ====
    // ==================

    fn get_pos_x() -> f32;

    fn get_pos_y() -> f32;

    fn get_rotation_radians() -> f32;

    // ==================
    // ===== CONFIG =====
    // ==================

    // fn get_config_max_rpm_left() -> u16;

    // fn get_config_max_rpm_right() -> u16;

    // fn get_config_max_rpm() -> u16;
    fn set_config_recommended_pwm(rpm: i16, pwm: i8);

    fn get_config_recommended_pwm(rpm: i16) -> i8;

    fn get_config_recommended_pos_min_rpm() -> i16;
    //
    fn get_config_recommended_pos_max_rpm() -> i16;

    fn get_config_recommended_neg_min_rpm() -> i16;
    //
    fn get_config_recommended_neg_max_rpm() -> i16;

    // set_config_pwm_to_rpm(0,0);
    // set_config_pwm_to_rpm(127, 500);
    // fn set_config_pwm_to_rpm(pwm: u32, rpm: u32);

}
