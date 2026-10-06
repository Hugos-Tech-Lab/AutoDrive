use anyhow::Result;
use esp_idf_hal::{
    adc::{
        Adc, AdcChannel, AdcUnit, attenuation::DB_12, oneshot::{
            AdcChannelDriver, AdcDriver, config::{AdcChannelConfig, Calibration},
        },
    }, gpio::ADCPin,
};

// Empirical correction against a multimeter at ~3.70 V battery voltage.
// ESP-IDF Curve calibration reads ~1.95 V where the ADC pin measures ~1.85 V.
const ADC_CALIBRATION_FACTOR: f32 = 1.0;
const NUM_SAMPLES: usize = 32;
const DIVIDER_RATIO: f32 = 2.0;

pub struct Battery<'a, C>
where
    C: AdcChannel,
{
    pin: AdcChannelDriver<
        'a,
        C,
        &'a AdcDriver<'a, C::AdcUnit>,
    >,
}

#[derive(Debug)]
pub struct BatteryReading {
    pub voltage: f32,
    pub percentage: u8,
}

impl<'a, C> Battery<'a, C>
where
    C: AdcChannel,
{
    pub fn new(
        adc: &'a AdcDriver<'a, C::AdcUnit>,
        pin: impl ADCPin<AdcChannel = C> + 'a,
    ) -> Result<Self> {
      
        let config = AdcChannelConfig {
            attenuation: DB_12,
            calibration: Calibration::Curve,
            ..Default::default()
        };

        let pin = AdcChannelDriver::new(
            adc,
            pin,
            &config,
        )?;

        Ok(Self { pin })
    }

    pub fn read(&mut self) -> Result<BatteryReading> {
        let mut total_mv = 0u32;

        for _ in 0..NUM_SAMPLES {
            let sample_mv = self.pin.read()? as u32;
            total_mv += sample_mv;
            dbg!(sample_mv);
        }

        let adc_mv = total_mv as f32 / NUM_SAMPLES as f32;
        let corrected_mv = adc_mv * ADC_CALIBRATION_FACTOR;
        let battery_voltage = corrected_mv * DIVIDER_RATIO / 1000.0;

        Ok(BatteryReading {
            voltage: battery_voltage,
            percentage: battery_percentage(battery_voltage),
        })
    }
}

fn battery_percentage(voltage: f32) -> u8 {
    if voltage >= 4.20 {
        100
    } else if voltage >= 4.15 {
        95
    } else if voltage >= 4.10 {
        90
    } else if voltage >= 4.05 {
        80
    } else if voltage >= 4.00 {
        70
    } else if voltage >= 3.95 {
        60
    } else if voltage >= 3.90 {
        50
    } else if voltage >= 3.85 {
        40
    } else if voltage >= 3.80 {
        30
    } else if voltage >= 3.75 {
        20
    } else if voltage >= 3.70 {
        15
    } else if voltage >= 3.65 {
        10
    } else if voltage >= 3.55 {
        5
    } else {
        0
    }
}