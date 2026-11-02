use esp_idf_svc::hal::units::*;
use esp_idf_svc::hal::{
    gpio::{AnyIOPin, Output, PinDriver},
    ledc::{LedcChannel, LedcDriver, LedcTimer, LedcTimerDriver, SpeedMode, config::TimerConfig},
};

struct MotorCalibration {
    speed_to_pwm: [i8; 256],
}

impl Default for MotorCalibration {
    fn default() -> Self {
        Self {
            speed_to_pwm: [0; 256],
        }
    }
}

impl MotorCalibration {
    pub fn get_pwm(&self, speed: i8) -> i8 {
        *self.speed_to_pwm.get(speed as usize).unwrap_or(&speed)
    }
}

pub struct Motor {
    power_driver: LedcDriver<'static>,
    directional_pin: PinDriver<'static, Output>,
    max_duty: u32,
    motor_calibration: MotorCalibration,
}

impl Motor {
    pub fn new<C, S, T>(
        directional_pin: AnyIOPin<'static>,
        power_pin: AnyIOPin<'static>,
        channel: C,
        timer: T,
    ) -> Self
    where
        C: LedcChannel<SpeedMode = S> + 'static,
        S: SpeedMode,
        T: LedcTimer<SpeedMode = S> + 'static,
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
            motor_calibration: MotorCalibration::default(),
        }
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.power_driver.set_duty(0)?;
        Ok(())
    }

    /// Percetange from -100 to 100
    pub fn set_speed(&mut self, speed: i8) -> anyhow::Result<()> {
        let pwm = self.motor_calibration.get_pwm(speed);
        self.set_pwm(pwm)
    }

    pub fn set_pwm(&mut self, pwm: i8) -> anyhow::Result<()> {
        let (power, reverse) = Self::convert(pwm, self.max_duty);
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

impl Drop for Motor {
    fn drop(&mut self) {
        if let Err(err) = self.stop() {
            log::error!("Could not stop motor on drop: '{err}'");
        }
    }
}
