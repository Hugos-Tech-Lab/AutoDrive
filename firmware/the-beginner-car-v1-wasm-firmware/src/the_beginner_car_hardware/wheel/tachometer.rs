use std::{
    num::NonZeroU32,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::{Duration, Instant},
};

use esp_idf_svc::hal::{
    gpio::{AnyIOPin, InterruptType, PinDriver, Pull},
    task::notification::Notification,
};
use log::info;

#[derive(Clone)]
pub struct Tachometer {
    rpm: Arc<AtomicU32>,
    pulses: Arc<AtomicU32>,
}

impl Tachometer {
    pub fn new(hall_sensor: AnyIOPin<'static>) -> Self {
        let pulses_per_rotation = 1 as f32;

        let rpm = Arc::new(AtomicU32::new(0));
        let pulses = Arc::new(AtomicU32::new(0));

        std::thread::Builder::new()
            .name("tachometer".to_string())
            .stack_size(4096)
            .spawn({
                let rpm = rpm.clone();
                let pulses = pulses.clone();

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
                        hall_sensor.enable_interrupt().unwrap();
                        if let Some(_) = notification.wait(10000) {
                            pulses.fetch_add(1, Ordering::Relaxed);
                            info!("detected");
                            let now = Instant::now();
                            if let Some(last_edge_time) = last_edge {
                                let elapsed = now.duration_since(last_edge_time);
                                if elapsed > Duration::from_millis(2) {
                                    let elapsed_secs = elapsed.as_secs_f32();
                                    let current_rpm =
                                        ((60.0 / elapsed_secs) / pulses_per_rotation) as u32;
                                    rpm.store(current_rpm, Ordering::Relaxed);
                                    info!("{}", current_rpm);
                                }
                            }
                            last_edge = Some(now);
                        } else {
                            rpm.store(0, Ordering::Relaxed);
                            last_edge = None;
                        }
                    }
                }
            })
            .unwrap();

        Self { rpm, pulses }
    }

    pub fn get_rpm(&self) -> u32 {
        self.rpm.load(Ordering::Relaxed)
    }
}
