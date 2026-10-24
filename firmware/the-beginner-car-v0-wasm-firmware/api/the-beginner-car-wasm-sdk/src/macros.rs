#[macro_export]
macro_rules! define_wasm_api {
    (
        $(
            $(#[$meta:meta])*
            fn $name:ident($($arg:ident: $ty:ty),*) $(-> $ret:ty)?;
        )*
    ) => {
        #[cfg(target_arch = "wasm32")]
        pub mod guest {
            #[link(wasm_import_module = "host")]
            unsafe extern "C" {
                $(
                    $(#[$meta])*
                    pub fn $name($($arg: $ty),*) $(-> $ret)?;
                )*
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        pub mod host {
            use core::ffi::c_void;

            pub struct FunctionDefinition {
                pub f_name: &'static str,
                pub f_pointer: *mut c_void,
            }

            pub trait WasmExports {
                $(
                    $(#[$meta])*
                    unsafe extern "C" fn $name(exec_env: *mut c_void, $($arg: $ty),*) $(-> $ret)?;
                )*
            }

            // Returns a static slice, removing the need for a Vec/Allocator
            pub fn host_functions<T: WasmExports>() -> &'static [FunctionDefinition] {
                &[
                    $(
                        FunctionDefinition {
                            f_name: stringify!($name),
                            f_pointer: T::$name as *const () as *mut c_void,
                        }
                    ),*
                ]
            }
        }
    };
}
