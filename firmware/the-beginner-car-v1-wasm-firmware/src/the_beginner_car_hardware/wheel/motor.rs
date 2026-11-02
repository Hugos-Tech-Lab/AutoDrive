use std::sync::Arc;
use std::sync::atomic::{AtomicI8, AtomicU32, Ordering};

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::units::*;
use esp_idf_svc::hal::{
    gpio::{AnyIOPin, Output, PinDriver},
    ledc::{LedcChannel, LedcDriver, LedcTimer, LedcTimerDriver, SpeedMode, config::TimerConfig},
};

use crate::the_beginner_car_hardware::wheel::tachometer::Tachometer;

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
    motor_calibration: MotorCalibration,
    tachometer: Tachometer,
    target_rpm: Arc<AtomicU32>,
    target_pwm: Arc<AtomicI8>,
}

impl Motor {
    pub fn new<C, S, T>(
        directional_pin: AnyIOPin<'static>,
        power_pin: AnyIOPin<'static>,
        channel: C,
        timer: T,
        tachometer: Tachometer,
    ) -> Self
    where
        C: LedcChannel<SpeedMode = S> + 'static,
        S: SpeedMode,
        T: LedcTimer<SpeedMode = S> + 'static,
    {
        let mut motor_hardware = MotorHardware::new(directional_pin, power_pin, channel, timer);

        let target_rpm = Arc::<AtomicU32>::new(AtomicU32::new(0));
        let target_pwm = Arc::new(AtomicI8::new(0));

        std::thread::Builder::new()
            .name("motor".to_string())
            .stack_size(4096)
            .spawn({
                let target_pwm = target_pwm.clone();
                let target_rpm = target_rpm.clone();
                let tachometer = tachometer.clone();
                move || {
                    let mut current_pwm: i32 = 0;
                    let mut control_counter = 0;

                    loop {
                        FreeRtos::delay_ms(1);

                        let target_rpm = target_rpm.load(Ordering::Relaxed);

                        if target_rpm == 0 {
                            current_pwm = 0;
                            let _ = motor_hardware.set_pwm(0);
                            control_counter = 0;
                            continue;
                        }

                        control_counter += 1;
                        if control_counter >= 50 {
                            control_counter = 0;

                            let actual = tachometer.get_rpm();

                            let error = target_rpm as i32 - actual as i32;

                            let kp = 0.15_f32;
                            let adjustment = (error as f32 * kp) as i32;

                            current_pwm += adjustment;
                            current_pwm = current_pwm.clamp(-127, 127);

                            let _ = motor_hardware.set_pwm(current_pwm as i8);
                        }
                    }
                }
            })
            .unwrap();
        Self {
            motor_calibration: MotorCalibration::default(),
            target_rpm,
            tachometer,
            target_pwm,
        }
    }
}

pub struct MotorHardware {
    power_driver: LedcDriver<'static>,
    directional_pin: PinDriver<'static, Output>,
    max_duty: u32,
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
        }
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.power_driver.set_duty(0)?;
        Ok(())
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

impl Drop for MotorHardware {
    fn drop(&mut self) {
        if let Err(err) = self.stop() {
            log::error!("Could not stop motor on drop: '{err}'");
        }
    }
}
