use esp_idf_svc::hal::{
    gpio::{AnyIOPin, Output, PinDriver},
    ledc::{LedcChannel, LedcDriver, LedcTimer, LedcTimerDriver, SpeedMode, config::TimerConfig},
};
use esp_idf_svc::hal::units::*;

pub struct Motor<'d> {
    power_driver: LedcDriver<'d>,
    directional_pin: PinDriver<'d, Output>,
    max_duty: u32,
}

impl<'d> Motor<'d> {
    pub fn new<C, S, T>(
        directional_pin: AnyIOPin<'d>,
        power_pin: AnyIOPin<'d>,
        channel: C,
        timer: T,
    ) -> Self
    where
        C: LedcChannel<SpeedMode = S> + 'd,
        S: SpeedMode,
        T: LedcTimer<SpeedMode = S> + 'd,
    {
        let directional_pin = PinDriver::output(directional_pin).unwrap();

        let timer_driver =
            LedcTimerDriver::new(timer, &TimerConfig::default().frequency(20.kHz().into()))
                .unwrap();

        let power_driver: LedcDriver<'_> =
            LedcDriver::new(channel, timer_driver, power_pin).unwrap();

        let max_duty = power_driver.get_max_duty();

        Self {
            directional_pin,
            power_driver,
            max_duty,
        }
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.power_driver.set_duty(0)?;
        Ok(())
    }

    pub fn set_speed(&mut self, speed: i8) -> anyhow::Result<()> {
        let (power, reverse) = Self::convert(speed, self.max_duty);
        if reverse {
            self.directional_pin.set_high().unwrap();
        } else {
            self.directional_pin.set_low().unwrap();
        }
        self.power_driver.set_duty(power).unwrap();

        Ok(())
    }

    fn convert(value: i8, max: u32) -> (u32, bool) {
        let negative = value < 0;
        let magnitude = value.unsigned_abs() as u32;

        let output = magnitude * max / 128;

        (output, negative)
    }
}
