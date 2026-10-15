/// For WASM guests!
///
/// Adds the imports
pub mod guest {
    #[link(wasm_import_module = "host")]
    unsafe extern "C" {
        /// takes u8 so we don't wait too long (i. 255ms max) between cancellation checks
        pub fn delay(milliseconds: u8);
        pub fn print(ptr: u32, len: u32);
    }
}

/// For the host to implement
pub mod host {
    use std::ffi::c_void;

    /// For the runtime to use
    pub struct FunctionDefinition {
        pub f_name: &'static str,
        pub f_pointer: *mut c_void,
    }

    pub fn host_functions<T: WasmUtils>() -> [FunctionDefinition; 2] {
        [
            FunctionDefinition {
                f_name: "delay",
                f_pointer: T::delay as *const () as *mut c_void,
            },
            FunctionDefinition {
                f_name: "print",
                f_pointer: T::print as *const () as *mut c_void,
            },
        ]
    }

    pub trait WasmUtils {
        unsafe extern "C" fn delay(exec_env: *mut c_void, milliseconds: u8);
        unsafe extern "C" fn print(exec_env: *mut c_void, ptr: u32, len: u32);
    }
}
