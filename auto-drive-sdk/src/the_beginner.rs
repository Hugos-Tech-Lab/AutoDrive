//! A physical small robot that's driven around the Tech Lab.
//! Use this crate to read its input / control it.

pub mod generated {
    wit_bindgen::generate!({
        world: "app",
        path: env!("GENERATED_WIT"),
        pub_export_macro: true,
        default_bindings_module: "::auto_drive_sdk::the_beginner::generated",
    });
}

pub use generated::hardware::control_wheel;
pub use generated::hardware::read_wheel;

pub trait TheBeginnerAutoDrive {
    /// Test
    fn run();
    fn run_2();
}

mod macros {
    #[macro_export]
    macro_rules! __the_beginner_register {
        ($app:ty) => {
            struct __TheBeginnerAdapter;

            impl $crate::the_beginner::generated::Guest for __TheBeginnerAdapter {
                fn run() {
                    <$app as $crate::the_beginner::TheBeginnerAutoDrive>::run();
                }

                fn run_2() {
                    <$app as $crate::the_beginner::TheBeginnerAutoDrive>::run_2();
                }
            }

            $crate::the_beginner::generated::export!(__TheBeginnerAdapter);
        };
    }
}

pub use crate::__the_beginner_register as register;
