use std::{ffi::c_void, thread};

use anyhow::Error;
use log::info;
use wamr_rust_sdk::{function::Function, instance::Instance, module::Module, runtime::Runtime};

use crate::hardware::Hardware;

#[derive(PartialEq)]
enum DeviceMode {
    Manual,
    Automatic,
}

#[unsafe(no_mangle)]
extern "C" fn control_wheel(wheel: f32, speed: f32) {
    println!("[WASM] wheel = {}", wheel);
    println!("[WASM] speed = {}", speed);
}

pub struct DeviceControl<'runtime> {
    hardware: Hardware,
    // runtime: Runtime,
    auto_execution: Option<Module<'runtime>>,
    mode: DeviceMode,
}

const BASIC_WASM: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/basic.wasm"));

impl<'runtime> DeviceControl<'runtime> {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            hardware: Hardware {},
            // runtime,
            auto_execution: None,
            mode: DeviceMode::Manual,
        })
    }

    pub fn activate_auto(&self) {
        // let thre = std::thread::Builder::new()
        //     .name("wasm".into())
        //     .stack_size(16 * 1024)
        //     .spawn(move || {
        unsafe {
            let watermark =
                esp_idf_sys::uxTaskGetStackHighWaterMark(esp_idf_sys::xTaskGetCurrentTaskHandle());

            let bytes = watermark as usize * core::mem::size_of::<esp_idf_sys::StackType_t>();

            log::info!("Stack high water mark A: {} bytes", bytes);
        }

        let thread = thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(move || {
                unsafe {
                    let watermark = esp_idf_sys::uxTaskGetStackHighWaterMark(
                        esp_idf_sys::xTaskGetCurrentTaskHandle(),
                    );

                    let bytes =
                        watermark as usize * core::mem::size_of::<esp_idf_sys::StackType_t>();

                    log::info!("Stack high water mark B: {} bytes", bytes);
                }

                // // // 1. Allocate a fixed static buffer block.
                // // // This reserves a clean 48KB continuous window that the system cannot fragment.
                println!("Free heap: {} bytes", unsafe {
                    esp_idf_sys::esp_get_free_heap_size()
                });

                // let isolated_pool: Vec<u8> = vec![0u8; 128 * 1024];

                println!("Free heap: {} bytes", unsafe {
                    esp_idf_sys::esp_get_free_heap_size()
                });
                // // // 2. Build the runtime pointing directly to your isolated buffer pool
                let runtime = Runtime::builder()
                    // .use_memory_pool(isolated_pool)
                    .use_system_allocator()
                    .register_host_function(
                        "delay",
                        crate::wasm::exposed_functions::delay as *mut c_void,
                    )
                    .register_host_function(
                        "print",
                        crate::wasm::exposed_functions::print as *mut c_void,
                    )
                    .register_host_function(
                        "set_onboard_led_color",
                        crate::wasm::exposed_functions::set_onboard_led_color as *mut c_void,
                    )
                    .build()
                    .unwrap();

                println!("Free heap: {} bytes", unsafe {
                    esp_idf_sys::esp_get_free_heap_size()
                });

                let module = Module::from_vec(&runtime, BASIC_WASM.to_vec(), "env").unwrap();

                let instance = Instance::new(&runtime, &module, 1024 * 32).unwrap();

                let function = Function::find_export_func(&instance, "main").unwrap();
                let params = vec![];

                info!("starting to run");

                let result = function.call(&instance, &params); // Change call to call_pthread
                let res = result.unwrap();
                info!("{:?}", res);
                // module.call.c(&instance, &params).unwrap();
                info!("stopped running");

                std::thread::sleep(std::time::Duration::from_secs(20));
            })
            .unwrap()
            .join();


        // println!("Free heap: {} bytes", unsafe {
        //     esp_idf_sys::esp_get_free_heap_size()
        // });

        //         let free = unsafe {
        //             esp_idf_sys::esp_get_free_heap_size()
        //         };

        //         println!("Free heap: {} bytes", free);

        // unsafe {
        //     let watermark = esp_idf_sys::uxTaskGetStackHighWaterMark(
        //         esp_idf_sys::xTaskGetCurrentTaskHandle(),
        //     );

        //     let bytes = watermark as usize * core::mem::size_of::<esp_idf_sys::StackType_t>();

        //     log::info!("Stack high water mark C: {} bytes", bytes);
        // }

        // // 4. Instantiate using the required 16KB Wasm stack size

        // // 5. Run the function

        // info!("Wasm returned: {:?}", result);
        // })
        // .unwrap();

        // thre.join().unwrap();
        // })
        // .unwrap();
    }

    pub fn control_wheel_manual(&self) {
        // let runtime = Runtime::new()?;

        // let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        // d.push("gcd_wasm32_wasi.wasm");
        // let mut module = Module::from_file(&runtime, d.as_path())?;

        if self.mode == DeviceMode::Automatic {
            // TODO: return error
        }

        //
        //
    }

    pub fn control_wheel_automatic(&self) {
        //
    }

    //
}
