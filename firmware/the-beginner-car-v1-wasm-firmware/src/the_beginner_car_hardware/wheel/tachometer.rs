use std::{
    num::NonZeroU32,
    sync::{
        Arc,
        atomic::{Atomic, AtomicBool, AtomicU32, Ordering},
    },
    time::{Duration, Instant},
};

use esp_idf_svc::hal::{
    gpio::{AnyIOPin, InterruptType, PinDriver, Pull},
    task::notification::Notification,
};
use log::warn;

pub struct Tachometer {
    rpm: Arc<AtomicU32>,
}

impl Tachometer {
    pub fn new(hall_sensor: AnyIOPin<'static>) -> Self {
        let rpm = Arc::new(AtomicU32::new(0));

        std::thread::Builder::new()
            .name("tachometer".to_string())
            .spawn({
                let rpm = rpm.clone();
                move || {
                    let notification = Arc::new(Notification::new());
                    let notifier = notification.notifier();
                    let mut hall_sensor = PinDriver::input(hall_sensor, Pull::Up).unwrap();
                    hall_sensor
                        .set_interrupt_type(InterruptType::NegEdge)
                        .unwrap();

                    unsafe {
                        hall_sensor
                            .subscribe({
                                let notifier = notifier.clone();

                                move || {
                                    notifier.notify(NonZeroU32::new(1).unwrap());
                                }
                            })
                            .unwrap();
                    }

                    let mut last_edge = None;
                    loop {
                        hall_sensor.enable_interrupt();
                        if let Some(_) = notification.wait(1000) {
                            let now = Instant::now();
                            if let Some(last_edge) = last_edge {
                                let elapsed = now.duration_since(last_edge);
                                if elapsed > Duration::from_millis(1) {
                                    let current_rpm = (60.0 / elapsed.as_secs_f32()) as u32;
                                    rpm.store(current_rpm, Ordering::Relaxed);
                                } else {
                                    warn!("motor spinning too fast");
                                }
                            }
                            last_edge = Some(now);
                        } else {
                            rpm.store(0, Ordering::Relaxed);
                            last_edge = None;
                        }
                    }
                }
            });

        Self { rpm }
    }

    pub fn get_rpm() {
        //
    }

    // pub fn wait_for_rotations(
    //     &mut self,
    //     target_rotations: u32,
    //     timeout: u32,
    //     ct: CancellationToken,
    // ) -> u32 {
    //     let timeout = Duration::from_millis(timeout as u64);
    //     let function_duration = Instant::now();

    //     if target_rotations == 0 {
    //         return 0;
    //     }

    //     let detected = Arc::new(AtomicBool::new(false));
    //     let hall_sensor = &mut self.hall_sensor;

    //     hall_sensor
    //         .set_interrupt_type(InterruptType::NegEdge)
    //         .unwrap();

    //     unsafe {
    //         hall_sensor
    //             .subscribe({
    //                 let detected = detected.clone();
    //                 move || {
    //                     detected.store(true, Ordering::Relaxed);
    //                 }
    //             })
    //             .unwrap();
    //     }

    //     hall_sensor.enable_interrupt().unwrap();

    //     // warm up get to first detected
    //     loop {
    //         if ct.is_cancelled() || function_duration.elapsed() >= timeout {
    //             break;
    //         }

    //         if detected.swap(false, Ordering::Relaxed) {
    //             break;
    //         }

    //         FreeRtos::delay_ms(1);
    //     }

    //     hall_sensor.enable_interrupt().unwrap();

    //     let target_start = Instant::now();

    //     let mut completed_rotations = 0u32;

    //     loop {
    //         if ct.is_cancelled() || function_duration.elapsed() >= timeout {
    //             break;
    //         }

    //         if detected.swap(false, Ordering::Relaxed) {
    //             hall_sensor.enable_interrupt().unwrap();
    //             completed_rotations += 1;
    //         }

    //         if completed_rotations >= target_rotations {
    //             break;
    //         }

    //         FreeRtos::delay_ms(1);
    //     }

    //     let target_duration = target_start.elapsed();
    //     let target_duration = target_duration.as_millis() as u32;

    //     hall_sensor.unsubscribe().unwrap();
    //     target_duration
    // }
}
