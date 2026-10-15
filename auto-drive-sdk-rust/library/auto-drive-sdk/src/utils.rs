use the_beginner_car_wasm_sdk::guest;


pub fn delay(milliseconds: u64) {
    let mut milliseconds = milliseconds;
    // convert from u64 -> u8
    unsafe {
        while milliseconds > u8::MAX as u64 {
            guest::delay(u8::MAX);
            milliseconds -= u8::MAX as u64;
        }

        if milliseconds > 0 {
            guest::delay(milliseconds as u8);
        }
    }
}

pub fn print(text: &str) {
    unsafe {
        guest::print(text.as_ptr() as u32, text.len() as u32);
    }
}
