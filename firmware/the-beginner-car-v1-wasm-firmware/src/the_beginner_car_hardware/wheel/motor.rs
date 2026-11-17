use std::sync::{Arc, Mutex};

use esp_idf_svc::hal::units::*;
use esp_idf_svc::hal::{
    gpio::{AnyIOPin, Output, PinDriver},
    ledc::{LedcChannel, LedcDriver, LedcTimer, LedcTimerDriver, SpeedMode, config::TimerConfig},
};

use crate::hardware_config::MotorConfig;
use crate::the_beginner_car_hardware::wheel::tachometer::Tachometer;

pub struct Motor {
    tachometer: Tachometer,
    motor_config: MotorConfig,
    motor_hardware: Arc<Mutex<MotorHardware>>,
}

impl Motor {
    pub fn new<C, S, T>(
        directional_pin: AnyIOPin<'static>,
        power_pin: AnyIOPin<'static>,
        channel: C,
        timer: T,
        tachometer: Tachometer,
        motor_config: MotorConfig,
    ) -> Self
    where
        C: LedcChannel<SpeedMode = S> + 'static,
        S: SpeedMode,
        T: LedcTimer<SpeedMode = S> + 'static,
    {
        let motor_hardware = Arc::new(Mutex::new(MotorHardware::new(
            directional_pin,
            power_pin,
            channel,
            timer,
        )));

        Self {
            tachometer,
            motor_config,
            motor_hardware,
        }
    }

    pub fn set_pwm(&self, pwm: i8) {
        let mut motor_hardware = self.motor_hardware.lock().unwrap();
        motor_hardware.set_pwm(pwm);
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        let mut motor_hardware = self.motor_hardware.lock().unwrap();
        motor_hardware.set_pwm(0);
        Ok(())
    }
}

pub struct MotorHardware {
    power_driver: LedcDriver<'static>,
    directional_pin: PinDriver<'static, Output>,
    max_duty: u32,
    pwm: i8,
}

impl MotorHardware {
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
            power_driver,
            directional_pin,
            max_duty,
            pwm: 0,
        }
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.power_driver.set_duty(0)?;
        Ok(())
    }

    pub fn set_pwm(&mut self, pwm: i8) -> anyhow::Result<()> {
        if pwm == self.pwm {
            return Ok(());
        }

        self.pwm = pwm;

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

impl Drop for MotorHardware {
    fn drop(&mut self) {
        if let Err(err) = self.stop() {
            log::error!("Could not stop motor on drop: '{err}'");
        }
    }
}
