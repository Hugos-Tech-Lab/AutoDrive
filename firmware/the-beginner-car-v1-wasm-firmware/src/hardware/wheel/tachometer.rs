use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use esp_idf_svc::hal::{
    delay::FreeRtos,
    gpio::{Input, InterruptType, PinDriver},
};

use crate::auto_script::cancellation_token::CancellationToken;

pub struct Tachometer {
    hall_sensor: PinDriver<'static, Input>,
}

impl Tachometer {
    pub fn wait_for_rotations(
        &mut self,
        target_rotations: u32,
        timeout: u32,
        ct: CancellationToken,
    ) -> u32 {
        let timeout = Duration::from_millis(timeout as u64);
        let function_duration = Instant::now();

        if target_rotations == 0 {
            return 0;
        }

        let detected = Arc::new(AtomicBool::new(false));
        let hall_sensor = &mut self.hall_sensor;

        hall_sensor
            .set_interrupt_type(InterruptType::NegEdge)
            .unwrap();

        unsafe {
            hall_sensor
                .subscribe({
                    let detected = detected.clone();
                    move || {
                        detected.store(true, Ordering::Relaxed);
                    }
                })
                .unwrap();
        }

        hall_sensor.enable_interrupt().unwrap();

        // warm up get to first detected
        loop {
            if ct.is_cancelled() || function_duration.elapsed() >= timeout {
                break;
            }

            if detected.swap(false, Ordering::Relaxed) {
                break;
            }

            FreeRtos::delay_ms(1);
        }

        hall_sensor.enable_interrupt().unwrap();

        let target_start = Instant::now();

        let mut completed_rotations = 0u32;

        loop {
            if ct.is_cancelled() || function_duration.elapsed() >= timeout {
                break;
            }

            if detected.swap(false, Ordering::Relaxed) {
                hall_sensor.enable_interrupt().unwrap();
                completed_rotations += 1;
            }

            if completed_rotations >= target_rotations {
                break;
            }

            FreeRtos::delay_ms(1);
        }

        let target_duration = target_start.elapsed();
        let target_duration = target_duration.as_millis() as u32;

        hall_sensor.unsubscribe().unwrap();
        target_duration
    }
}
