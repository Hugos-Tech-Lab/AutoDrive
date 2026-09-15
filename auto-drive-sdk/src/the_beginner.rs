//! A physical small robot that's driven around the Tech Lab.
//! Use this crate to read its input / control it.

mod ffi {
    #[link(wasm_import_module = "host")]
    unsafe extern "C" {
        pub fn delay(milliseconds: u64);
        pub fn print(text: u32);
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

pub trait TheBeginnerAutoDrive {
    fn run();
}

mod macros {
    #[macro_export]
    macro_rules! register {
        ($app:ty) => {
            #[unsafe(export_name = "run")]
            pub extern "C" fn run() -> () {
                <$app as $crate::the_beginner::TheBeginnerAutoDrive>::run()
            }

            #[cfg(not(test))]
            #[panic_handler]
            fn panic(_info: &core::panic::PanicInfo) -> ! {
                loop {}
            }
        };
    }
}
