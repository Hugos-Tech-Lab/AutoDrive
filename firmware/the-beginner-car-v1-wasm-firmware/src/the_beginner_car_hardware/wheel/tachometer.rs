use std::{
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    thread,
    time::Duration,
};

use esp_idf_svc::hal::{
    gpio::AnyIOPin,
    pcnt::{
        PcntUnitDriver,
        config::{ChannelEdgeAction, UnitConfig},
    },
};

pub struct Tachometer {
    rpm: Arc<AtomicU32>,
}

impl Tachometer {
    pub fn new(hall_sensor: AnyIOPin<'static>) -> anyhow::Result<Self> {
        let pulses_per_rotation: u32 = 1;
        let poll_interval = Duration::from_millis(100);

        let rpm = Arc::new(AtomicU32::new(0));
        let thread_rpm = rpm.clone();

        let mut pcnt = PcntUnitDriver::new(&UnitConfig {
            low_limit: -1,
            high_limit: 32000,
            ..Default::default()
        })?;

        pcnt.add_channel(Some(hall_sensor), None::<AnyIOPin>, &Default::default())?
            .set_edge_action(ChannelEdgeAction::Hold, ChannelEdgeAction::Increase)?;

        pcnt.enable()?;
        pcnt.start()?;

        thread::Builder::new()
            .name("tachometer_poll".to_string())
            .stack_size(2048)
            .spawn(move || {
                let rpm = thread_rpm;

                let poll_interval_secs = poll_interval.as_secs_f32();
                let intervals_per_minute = 60.0 / poll_interval_secs;

                let ppr = pulses_per_rotation as f32;

                loop {
                    thread::sleep(poll_interval);

                    if let Ok(count) = pcnt.get_count() {
                        pcnt.clear_count().ok();

                        let total_pulses_per_min = (count as f32) * intervals_per_minute;

                        let current_rpm = (total_pulses_per_min / ppr) as u32;

                        rpm.store(current_rpm, Ordering::Relaxed);
                    }
                }
            })?;

        Ok(Self { rpm })
    }

    pub fn get_rpm(&self) -> u32 {
        self.rpm.load(Ordering::Relaxed)
    }
}
