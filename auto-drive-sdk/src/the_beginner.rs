//! A physical small robot that's driven around the Tech Lab. 
//! Use this crate to read it's input / control it.

pub mod generated {
    wit_bindgen::generate!({
        world: "app",
        path: env!("GENERATED_WIT"),
        pub_export_macro: true,
        default_bindings_module: "::auto_drive_the_beginner_sdk::generated",
    });
}

pub use generated::hardware::control_wheel as control_wheel;
pub use generated::hardware::read_wheel as read_wheel;

#[macro_export]
macro_rules! register {
    ($app:ty) => {
        struct __TheBeginnerAdapter;

        impl $crate::generated::Guest for __TheBeginnerAdapter {
            fn run() {
                <$app as $crate::the_beginner_auto_drive::TheBeginnerAutoDrive>::run();
            }

            fn run_2() {
                <$app as $crate::the_beginner_auto_drive::TheBeginnerAutoDrive>::run_2();
            }
        }

        $crate::generated::export!(__TheBeginnerAdapter);
    };
}
