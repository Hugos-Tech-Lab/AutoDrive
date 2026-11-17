use std::ffi::CStr;

use esp_idf_sys::{
    ESP_OK, MALLOC_CAP_8BIT, MALLOC_CAP_INTERNAL, TaskStatus_t, UBaseType_t,
    esp_get_free_heap_size, esp_get_minimum_free_heap_size, esp_ota_get_running_partition,
    esp_partition_get_sha256, heap_caps_get_free_size, heap_caps_get_largest_free_block,
    uxTaskGetNumberOfTasks, uxTaskGetSystemState,
};

pub struct HardwareInfo {
    pub fw_sha256: [u8; 32],
}

impl HardwareInfo {
    pub fn new() -> Result<Self, ()> {
        let mut fw_sha256 = [0u8; 32];
        unsafe {
            let running_part = esp_ota_get_running_partition();
            if !running_part.is_null() {
                if esp_partition_get_sha256(running_part, fw_sha256.as_mut_ptr()) != ESP_OK {
                    return Err(());
                }
            }
        }

        Ok(Self { fw_sha256 })
    }
}

pub fn get_hardware_info() {}
pub fn get_fw_sha256() -> Option<[u8; 32]> {
    None
}

pub fn sample_cpu_usage() {
    let num_tasks = unsafe { uxTaskGetNumberOfTasks() } as usize + 4;
    let mut task_array: Vec<TaskStatus_t> = Vec::with_capacity(num_tasks);
    let mut total_run_time: u32 = 0;

    let recorded_tasks = unsafe {
        let count = uxTaskGetSystemState(
            task_array.as_mut_ptr(),
            num_tasks as UBaseType_t,
            &mut total_run_time,
        );
        task_array.set_len(count as usize);
        count as usize
    };

    if recorded_tasks == 0 || total_run_time == 0 {
        println!(
            "Runtime stats unavailable. Did you enable CONFIG_FREERTOS_GENERATE_RUN_TIME_STATS?"
        );
        return;
    }

    // Prevent division by zero
    let total_time_divisor = total_run_time / 100;
    if total_time_divisor == 0 {
        return;
    }

    println!("--- Task CPU Usage (Cumulative since boot) ---");
    for task in &task_array {
        let name = unsafe { CStr::from_ptr(task.pcTaskName) }
            .to_str()
            .unwrap_or("?");
        let percent = task.ulRunTimeCounter / total_time_divisor;
        if name.starts_with("IDLE") {
            let load = 100u32.saturating_sub(percent);
            println!(
                "[Core Load] {} -> {}% active ({}% idle)",
                name, load, percent
            );
        } else if percent > 0 {
            println!(
                "  Task {:<16}: {}% (ticks: {})",
                name, percent, task.ulRunTimeCounter
            );
        }
    }
}

pub fn print_task_stats() {
    let num_tasks = unsafe { uxTaskGetNumberOfTasks() } as usize + 4;
    let mut task_array: Vec<TaskStatus_t> = Vec::with_capacity(num_tasks);
    let mut total_run_time: u32 = 0;

    let recorded_tasks = unsafe {
        let count = uxTaskGetSystemState(
            task_array.as_mut_ptr(),
            num_tasks as UBaseType_t,
            &mut total_run_time,
        );
        task_array.set_len(count as usize);
        count as usize
    };

    if recorded_tasks == 0 || total_run_time == 0 {
        return;
    }

    let total_time_divisor = (total_run_time / 100).max(1);

    println!(
        "{:<16} | {:>6} | {:>14} | {:>10}",
        "Task", "CPU %", "Min Free Stack", "Stack Base"
    );
    println!("{}", "-".repeat(56));

    for task in &task_array {
        let name = unsafe { CStr::from_ptr(task.pcTaskName) }
            .to_str()
            .unwrap_or("?");
        let cpu_percent = task.ulRunTimeCounter / total_time_divisor;

        // `usStackHighWaterMark` is the minimum free stack space ever reached (in bytes)
        let min_free_bytes = task.usStackHighWaterMark;

        // `pxStackBase` is the lowest memory address of this task's stack
        let stack_base = task.pxStackBase as usize;

        println!(
            "{:<16} | {:>5}% | {:>10} B   | 0x{:08x}",
            name, cpu_percent, min_free_bytes, stack_base
        );
    }
}

pub fn print_unallocated_memory() {
    unsafe {
        // Total free heap across all memory regions
        let total_free = esp_get_free_heap_size();

        // Lowest free heap ever reached since boot (heap high-water mark)
        let min_ever_free = esp_get_minimum_free_heap_size();

        // Free internal byte-addressable RAM (where FreeRTOS task stacks live)
        let free_internal = heap_caps_get_free_size(MALLOC_CAP_INTERNAL | MALLOC_CAP_8BIT);

        // Largest single contiguous block available (the max stack size a new thread can request)
        let max_new_stack = heap_caps_get_largest_free_block(MALLOC_CAP_INTERNAL | MALLOC_CAP_8BIT);

        println!(
            "Total Free Heap:        {} bytes ({:.1} KB)",
            total_free,
            total_free as f32 / 1024.0
        );
        println!(
            "Min Free Heap Ever:     {} bytes ({:.1} KB)",
            min_ever_free,
            min_ever_free as f32 / 1024.0
        );
        println!(
            "Free Internal RAM:      {} bytes ({:.1} KB)",
            free_internal,
            free_internal as f32 / 1024.0
        );
        println!(
            "Max New Stack Block:    {} bytes ({:.1} KB)",
            max_new_stack,
            max_new_stack as f32 / 1024.0
        );
    }
}
