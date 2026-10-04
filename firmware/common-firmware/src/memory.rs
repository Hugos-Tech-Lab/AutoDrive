pub fn print_heap() {
    let mut multi_heap_info = esp_idf_sys::multi_heap_info_t {
        total_free_bytes: 0,
        total_allocated_bytes: 0,
        largest_free_block: 0,
        minimum_free_bytes: 0,
        allocated_blocks: 0,
        free_blocks: 0,
        total_blocks: 0,
    };

    unsafe {
        esp_idf_sys::heap_caps_get_info(&mut multi_heap_info, esp_idf_sys::MALLOC_CAP_8BIT);
    }

    println!("hea (current): {:?}", multi_heap_info);
}

pub fn print_stack() {
    let remaining;
    unsafe {
        remaining = esp_idf_sys::uxTaskGetStackHighWaterMark(std::ptr::null_mut());
    }

    println!("stack (high watermark): {:?}", remaining);
}
