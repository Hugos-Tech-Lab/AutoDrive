use std::sync::Arc;

use esp_idf_svc::hal::{
    gpio::AnyIOPin,
    pcnt::{
        PcntUnitDriver,
        config::{ChannelConfig, ChannelEdgeAction, ChannelLevelAction, UnitConfig},
    },
};

#[derive(Clone)]
pub struct Tachometer {
    pcnt_unit: Arc<PcntUnitDriver<'static>>,
    pulses_per_rotation: u32,
    wheel_circumference_cm: f32,
}

impl Tachometer {
    pub fn new(
        hall_sensor_a: AnyIOPin<'static>,
        hall_sensor_b: AnyIOPin<'static>,
        wheel_circumference_cm: f32,
        pulses_per_rotation: u32,
    ) -> Self {
        let pulses_per_rotation = pulses_per_rotation * 4;

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

        Self {
            // rpm,
            pulses_per_rotation,
            pcnt_unit,
            wheel_circumference_cm,
        }
    }

    pub fn get_total_pulses(&self) -> i32 {
        self.pcnt_unit.get_count().unwrap()
    }

    pub fn get_pulses_per_rotation(&self) -> u32 {
        self.pulses_per_rotation
    }

    pub fn get_wheel_circumference_cm(&self) -> f32 {
        self.wheel_circumference_cm
    }
}
