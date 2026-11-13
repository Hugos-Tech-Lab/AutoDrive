use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering},
    },
    time::{Duration, Instant},
};

use esp_idf_svc::hal::{
    gpio::AnyIOPin,
    pcnt::{
        PcntUnitDriver,
        config::{ChannelConfig, ChannelEdgeAction, ChannelLevelAction, UnitConfig},
    },
};

#[derive(Clone)]
pub struct Tachometer {
    rpm: Arc<AtomicU32>,
    pcnt_unit: Arc<PcntUnitDriver<'static>>,
    cm_per_pulse: f32,
}

impl Tachometer {
    pub fn new(
        hall_sensor_a: AnyIOPin<'static>,
        hall_sensor_b: AnyIOPin<'static>,
        wheel_circumference_cm: f32,
        pulses_per_rotation: i32,
    ) -> Self {
        let pulses_per_rotation_4x = (pulses_per_rotation as f32) * 4.0;

        let rpm = Arc::new(AtomicU32::new(0));
        let pulses = Arc::new(AtomicU32::new(0));

        let mut pcnt_unit: PcntUnitDriver<'static> = PcntUnitDriver::new(&UnitConfig {
            low_limit: -32768,
            high_limit: 32767,
            ..Default::default()
        })
        .unwrap();

        // Bitwise copy to route the same physical pins to both channels in the GPIO matrix
        let pin_a_ch0 = unsafe { core::ptr::read(&hall_sensor_a) };
        let pin_b_ch0 = unsafe { core::ptr::read(&hall_sensor_b) };
        let pin_a_ch1 = hall_sensor_a;
        let pin_b_ch1 = hall_sensor_b;

        // Scope the channels so their `&mut pcnt_unit` borrows are released before enable/start
        {
            // Channel 0: Phase A signal, Phase B control
            let ch0 = pcnt_unit
                .add_channel(Some(pin_a_ch0), Some(pin_b_ch0), &ChannelConfig::default())
                .unwrap();
            ch0.set_edge_action(ChannelEdgeAction::Decrease, ChannelEdgeAction::Increase)
                .unwrap();
            ch0.set_level_action(ChannelLevelAction::Keep, ChannelLevelAction::Inverse)
                .unwrap();

            // Channel 1: Phase B signal, Phase A control
            let ch1 = pcnt_unit
                .add_channel(Some(pin_b_ch1), Some(pin_a_ch1), &ChannelConfig::default())
                .unwrap();
            ch1.set_edge_action(ChannelEdgeAction::Decrease, ChannelEdgeAction::Increase)
                .unwrap();
            ch1.set_level_action(ChannelLevelAction::Inverse, ChannelLevelAction::Keep)
                .unwrap();
        }

        pcnt_unit.enable().unwrap();
        pcnt_unit.clear_count().unwrap();
        pcnt_unit.start().unwrap();

        let pcnt_unit = Arc::new(pcnt_unit);

        std::thread::Builder::new()
            .name("tachometer".to_string())
            .stack_size(4096)
            .spawn({
                let pcnt_unit = pcnt_unit.clone();
                let rpm = rpm.clone();

                move || {
                    let mut last_count = pcnt_unit.get_count().unwrap_or(0) as i16;
                    let mut last_time = Instant::now();

                    loop {
                        std::thread::sleep(Duration::from_millis(100));

                        let now = Instant::now();
                        let current_count = pcnt_unit.get_count().unwrap_or(0) as i16;
                        let elapsed = now.duration_since(last_time).as_secs_f32();

                        // Cast to i16 so wrapping_sub cleanly handles the [-32768, 32767] hardware rollover
                        let delta = current_count.wrapping_sub(last_count);
                        let abs_delta = delta.unsigned_abs() as u32;

                        let current_rpm = if elapsed > 0.0 {
                            ((abs_delta as f32 / pulses_per_rotation_4x) / (elapsed / 60.0)) as u32
                        } else {
                            0
                        };

                        rpm.store(current_rpm, Ordering::Relaxed);

                        last_count = current_count;
                        last_time = now;
                    }
                }
            })
            .unwrap();

        Self {
            rpm,
            cm_per_pulse: wheel_circumference_cm / pulses_per_rotation_4x,
            pcnt_unit,
        }
    }

    pub fn get_rpm(&self) -> u32 {
        self.rpm.load(Ordering::Relaxed)
    }

    pub fn get_total_pulses(&self) -> i32 {
        self.pcnt_unit.get_count().unwrap()
    }

    pub fn cm_per_pulse(&self) -> f32 {
        self.cm_per_pulse
    }
}
