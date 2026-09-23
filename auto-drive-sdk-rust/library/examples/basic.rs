// #![no_std]

// use auto_drive_sdk::the_beginner::{self, TheBeginnerAutoDrive, register};


// // Tell the compiler to use a lightweight panic handler
// #[panic_handler]
// fn panic(_info: &core::panic::PanicInfo) -> ! {
//     loop {}
// }

// // Low-level HOST call: Tell the ESP32 to execute something
// unsafe extern "C" {
//     fn host_log_status(status: i32);
// }

// // Low-level GUEST call: The ESP32 calls this to trigger "auto"
// #[unsafe(no_mangle)]
// pub extern "C" fn run() -> i32 {
//     1 // Return a simple MVP primitive type (i32, i64, f32, f64)
// }

// // struct BasicAutoDrive;

// // impl TheBeginnerAutoDrive for BasicAutoDrive {
// //     fn run() {
// //         the_beginner::control_wheel(1.0, 2.0);
// //     }
    
// //     fn run_2() -> () {
// //         todo!()
// //     }
// // }

// // register!(BasicAutoDrive);
