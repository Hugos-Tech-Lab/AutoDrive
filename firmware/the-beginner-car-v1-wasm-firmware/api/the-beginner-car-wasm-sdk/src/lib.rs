#![no_std]

pub mod macros;

// Invoke the macro to generate the guest module and host trait
define_wasm_api! {
    /// takes u8 so we don't wait too long (i.e. 255ms max) between cancellation checks
    fn delay(milliseconds: u8);

    fn delay_for_motor_left_rotations(count: u32, timeout: u32) -> u32;

    fn delay_for_motor_right_rotations(count: u32, timeout: u32) -> u32;

    /// prints from memory
    fn print(ptr: u32, len: u32);

    /// Sets the onboard LED color
    fn set_onboard_led_color(r: u8, g: u8, b: u8);

    // This is a % from -100 to 100
    fn set_motor_left_speed(speed: i8);

    // This is a % from -100 to 100
    fn set_motor_right_speed(speed: i8);

    // From -128 to 127
    fn set_motor_left_pwm(pwm: i8);

    // From -128 to 127
    fn set_motor_right_pwm(pwm: i8);

    fn set_led_left_on();

    fn set_led_left_off();

    fn set_led_right_on();

    fn set_led_right_off();

    fn get_ir_front_far_left_value() -> u16;

    fn get_ir_front_left_value() -> u16;

    fn get_ir_front_center_value() -> u16;

    fn get_ir_front_right_value() -> u16;

    fn get_ir_front_far_right_value() -> u16;

    fn get_ir_back_center_value() -> u16;

    fn get_ir_back_left_value() -> u16;
}
