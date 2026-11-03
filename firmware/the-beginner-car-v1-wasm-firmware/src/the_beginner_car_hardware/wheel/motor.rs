use std::sync::Arc;
use std::sync::atomic::{AtomicI8, AtomicI32, Ordering};

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::units::*;
use esp_idf_svc::hal::{
    gpio::{AnyIOPin, Output, PinDriver},
    ledc::{LedcChannel, LedcDriver, LedcTimer, LedcTimerDriver, SpeedMode, config::TimerConfig},
};
use log::info;

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

#[derive(Debug, Default)]
pub struct RpmToPwm {
    sorted_rpm_to_pwm: Vec<(i32, i8)>,
}

impl RpmToPwm {
    pub fn get_pwm(&self, rpm: i32) -> i8 {
        let sorted_rpm_to_pwm = &self.sorted_rpm_to_pwm;

        let index = match sorted_rpm_to_pwm.binary_search_by_key(&rpm, |p| p.0) {
            Ok(i) => return sorted_rpm_to_pwm[i].1,
            Err(i) => i,
        };

        if index == 0 {
            return sorted_rpm_to_pwm[0].1;
        }

        if index >= sorted_rpm_to_pwm.len() {
            return sorted_rpm_to_pwm[sorted_rpm_to_pwm.len() - 1].1;
        }

        let (rpm1, pwm1) = sorted_rpm_to_pwm[index - 1];
        let (rpm2, pwm2) = sorted_rpm_to_pwm[index];

        let ratio = (rpm - rpm1) as f32 / (rpm2 - rpm1) as f32;

        (pwm1 as f32 + ratio * (pwm2 - pwm1) as f32).round() as i8
    }
}

pub struct Motor {
    min_forward_rpm: i32, // typically: 10
    max_forward_rpm: i32, // typically: 200
    rpm_to_pwm: RpmToPwm,
    tachometer: Tachometer,
    target_rpm: Arc<AtomicI32>,
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

        let target_rpm = Arc::new(AtomicI32::new(0));
        let target_pwm = Arc::new(AtomicI8::new(0));
        let rpm_to_pwm = RpmToPwm {
            sorted_rpm_to_pwm: vec![(60, 20), (260, 127)],
        };
        std::thread::Builder::new()
            .name("motor".to_string())
            .stack_size(4096)
            .spawn({
                let target_pwm = target_pwm.clone();
                let target_rpm = target_rpm.clone();
                let tachometer = tachometer.clone();
                move || {
                    let mut current_pwm: i32 = 105;
                    let mut control_counter = 0;
                    let mut last_rpm_target: i32 = 0;
                    let mut grace_cycles: i32 = 0;

                    loop {
                        FreeRtos::delay_ms(1);
                        let target = target_rpm.load(Ordering::Relaxed);

                        if target != last_rpm_target {
                            last_rpm_target = target;
                            grace_cycles = 4;
                        }

                        if target == 0 {
                            current_pwm = 0;
                            let _ = motor_hardware.set_pwm(0);
                            control_counter = 0;
                            continue;
                        }

                        control_counter += 1;
                        if control_counter >= 50 {
                            control_counter = 0;

                            let actual = tachometer.get_rpm() as i32;

                            let is_forward_target = target > 0;

                            if grace_cycles > 0 {
                                grace_cycles -= 1;

                                let pwm = rpm_to_pwm.get_pwm(current_pwm);
                                let _ = motor_hardware.set_pwm(pwm);
                                continue;
                            }

                            let effective_actual = if is_forward_target { actual } else { -actual };

                            let error = target - effective_actual;

                            let kp = 0.10_f32;
                            let adjustment = (error as f32 * kp) as i32;

                            current_pwm += adjustment;

                            if is_forward_target {
                                current_pwm = current_pwm.clamp(20, 127); // Forward only
                            } else {
                                current_pwm = current_pwm.clamp(-127, 20); // Reverse only
                            }

                            let _ = motor_hardware.set_pwm(current_pwm as i8);
                        }
                    }
                }
            })
            .unwrap();

        Self {
            tachometer,
            min_forward_rpm: 60,
            max_forward_rpm: 260,
            rpm_to_pwm: RpmToPwm {
                sorted_rpm_to_pwm: vec![(60, 0), (260, 127)],
            },
            target_rpm,
            target_pwm,
        }
    }

    pub fn set_speed(&self, speed: i8) {
        let speed = speed.clamp(-100, 100);

        let rpm = if speed == 0 {
            0
        } else {
            let magnitude = speed.abs() as i32;

            let rpm = self.min_forward_rpm
                + (self.max_forward_rpm - self.min_forward_rpm) * (magnitude - 1) / 99;

            rpm * speed.signum() as i32
        };

        self.set_rpm(rpm);
    }

    pub fn set_rpm(&self, rpm: i32) {
        info!("{}", rpm);
        self.target_rpm.store(rpm, Ordering::Relaxed);
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        self.target_rpm.store(0, Ordering::Relaxed);
        self.target_pwm.store(0, Ordering::Relaxed);
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

        // info!("{}", pwm);

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
