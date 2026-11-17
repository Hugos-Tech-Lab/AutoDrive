use std::{ffi::CStr, io, thread::JoinHandle};

use esp_idf_svc::hal::task::thread::ThreadSpawnConfiguration;

pub fn spawn_thread<F, T>(
    name: &'static CStr,
    priority: u8,
    stack_size: usize,
    f: F,
) -> io::Result<JoinHandle<T>>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    ThreadSpawnConfiguration {
        // good to set name here as well as thread builder for statistics
        name: Some(name),
        priority,
        ..Default::default()
    }
    .set()
    .unwrap();

    let res = std::thread::Builder::new()
        // good to set name here as well as thread spawn configuration for the rust panics
        .name(name.to_str().unwrap().to_string())
        // good to set stack size here and not in configuration because it works when you set it here
        .stack_size(stack_size)
        .spawn(f);

    ThreadSpawnConfiguration::default().set().unwrap();

    res
}
