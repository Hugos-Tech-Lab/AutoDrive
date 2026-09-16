//! A physical small robot that's driven around the Tech Lab.
//! Use this crate to read its input / control it.

mod ffi {
    #[link(wasm_import_module = "host")]
    unsafe extern "C" {
        pub fn delay(milliseconds: u64);
        pub fn print(text: u32);
        pub fn set_onboard_led_color(r: u8, g: u8, b: u8);
    }
}

pub fn delay(milliseconds: u64) {
    unsafe {
        ffi::delay(milliseconds);
    }
}

pub fn print(text: u32) {
    unsafe {
        ffi::print(text);
    }
}

pub fn set_onboard_led_color(r: u8, g: u8, b: u8) {
    unsafe {
        ffi::set_onboard_led_color(r, g, b);
    }
}
