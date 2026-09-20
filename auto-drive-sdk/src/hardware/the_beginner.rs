//! A physical small robot that's driven around the Tech Lab.
//! Use this crate to read its input / control it.

mod ffi {
    #[link(wasm_import_module = "host")]
    unsafe extern "C" {
        /// takes u8 so we don't wait too long (i. 255ms max) between cancellation checks
        pub fn delay(milliseconds: u8);
        pub fn print(text: u32);
        pub fn set_onboard_led_color(r: u8, g: u8, b: u8);
    }
}

pub fn delay(milliseconds: u64) {
    let mut milliseconds = milliseconds;
    // convert from u64 -> u8
    unsafe {
        while milliseconds > u8::MAX as u64 {
            ffi::delay(u8::MAX);
            milliseconds -= u8::MAX as u64;
        }

        if milliseconds > 0 {
            ffi::delay(milliseconds as u8);
        }
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
