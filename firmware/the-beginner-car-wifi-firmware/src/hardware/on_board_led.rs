use std::{
    borrow::{Borrow, BorrowMut},
    sync::{Mutex, OnceLock},
};

use esp_idf_svc::hal::{
    gpio::{AnyIOPin, Gpio9, Gpio10, Gpio11, Gpio12, Gpio38, Output, PinDriver},
    ledc::{config::TimerConfig, CHANNEL0, CHANNEL1, LedcDriver, LedcTimerDriver, TIMER0, TIMER1},
    spi::{self, Dma, SpiBusDriver, SpiConfig, SpiDriver, SpiDriverConfig},
    units::Hertz,
};
use esp_idf_svc::hal::units::*;
use smart_leds_trait::{RGB8, SmartLedsWrite};
use ws2812_spi::{devices, Ws2812};

// We still need this holder to satisfy the WS2812 driver's borrow traits
pub struct SpiDriverHolder {
    spi_driver: SpiDriver<'static>,
}

impl Borrow<SpiDriver<'static>> for SpiDriverHolder {
    fn borrow(&self) -> &SpiDriver<'static> {
        &self.spi_driver
    }
}

impl BorrowMut<SpiDriver<'static>> for SpiDriverHolder {
    fn borrow_mut(&mut self) -> &mut SpiDriver<'static> {
        &mut self.spi_driver
    }
}

// Bundle all our active hardware drivers into one state struct
struct HardwareState {
    led: Ws2812<SpiBusDriver<'static, SpiDriverHolder>, devices::Ws2812>,
    motor_a_dir: PinDriver<'static, Output>, // Removed Gpio9
    motor_a_pwm: LedcDriver<'static>,
    motor_a_max_duty: u32,
    motor_b_dir: PinDriver<'static, Output>, // Removed Gpio11
    motor_b_pwm: LedcDriver<'static>,
    motor_b_max_duty: u32,
}

// yeee hawwhh
unsafe impl Send for HardwareState {}

// The global, mutex-protected hardware state
static HARDWARE_STATE: OnceLock<Mutex<HardwareState>> = OnceLock::new();


pub struct Hardware;

impl Hardware {
    pub fn new(
        pin_38: Gpio38<'static>,
        spi_2: spi::SPI2<'static>,
        pin_9: Gpio9<'static>,
        pin_10: Gpio10<'static>,
        pin_11: Gpio11<'static>,
        pin_12: Gpio12<'static>,
        timer0: TIMER0<'static>,
        timer1: TIMER1<'static>,
        channel0: CHANNEL0<'static>,
        channel1: CHANNEL1<'static>,
    ) -> Self {
        // --- Setup Motors ---
        let mut motor_a_dir_pin = PinDriver::output(pin_9).unwrap();
        let motor_a_timer_driver =
            LedcTimerDriver::new(timer0, &TimerConfig::default().frequency(20.kHz().into()))
                .unwrap();
        let mut motor_a_pwm_driver =
            LedcDriver::new(channel0, motor_a_timer_driver, pin_12).unwrap();

        let mut motor_b_dir_pin = PinDriver::output(pin_11).unwrap();
        let motor_b_timer_driver =
            LedcTimerDriver::new(timer1, &TimerConfig::default().frequency(20.kHz().into()))
                .unwrap();
        let mut motor_b_pwm_driver =
            LedcDriver::new(channel1, motor_b_timer_driver, pin_10).unwrap();

        // Start stopped
        motor_a_dir_pin.set_low().unwrap();
        motor_b_dir_pin.set_low().unwrap();

        let motor_a_max_duty = motor_a_pwm_driver.get_max_duty();
        let motor_b_max_duty = motor_b_pwm_driver.get_max_duty();

        motor_a_pwm_driver.set_duty(0).unwrap();
        motor_b_pwm_driver.set_duty(0).unwrap();

        // --- Setup LED ---
        let bus_config = SpiDriverConfig::new().dma(Dma::Auto(4096));
        let spi_driver = SpiDriver::new_without_sclk(
            spi_2,
            pin_38,
            Option::<AnyIOPin>::None,
            &bus_config,
        )
        .unwrap();

        let spi_driver = SpiDriverHolder { spi_driver };
        let spi_config = SpiConfig::new().baudrate(Hertz(3_200_000)).write_only(true);
        let spi_bus_driver = SpiBusDriver::new(spi_driver, &spi_config).unwrap();
        
        let mut led = Ws2812::<_, devices::Ws2812>::new(spi_bus_driver);
        
        // Start with LED off
        led.write([RGB8 { r: 0, g: 0, b: 0 }]).unwrap();

        // Construct the state payload
        let state = HardwareState {
            led,
            motor_a_dir: motor_a_dir_pin,
            motor_a_pwm: motor_a_pwm_driver,
            motor_a_max_duty,
            motor_b_dir: motor_b_dir_pin,
            motor_b_pwm: motor_b_pwm_driver,
            motor_b_max_duty,
        };

        // Save it into the global Mutex
        if HARDWARE_STATE.set(Mutex::new(state)).is_err() {
            panic!("Hardware state already initialized");
        }

        Self
    }

    fn convert(value: i8, max: u32) -> (u32, bool) {
        let negative = value < 0;
        let magnitude = value.unsigned_abs() as u32;

        // Example mapping: -128..=127 → 0..=max
        let output = magnitude * max / 128;

        (output, negative)
    }

    pub fn set_color(color: RGB8) {
        let mut state = HARDWARE_STATE.get().expect("Hardware not initialized").lock().unwrap();
        state.led.write([color]).unwrap();
    }

    pub fn off() {
        let mut state = HARDWARE_STATE.get().expect("Hardware not initialized").lock().unwrap();
        state.led.write([RGB8 { r: 0, g: 0, b: 0 }]).unwrap();
    }

    pub fn set_motor_a_speed(speed: i8) {
        let mut state = HARDWARE_STATE.get().expect("Hardware not initialized").lock().unwrap();
        
        let (power, reverse) = Self::convert(speed, state.motor_a_max_duty);
        if reverse {
            state.motor_a_dir.set_high().unwrap();
            println!("setting motor a direction high");
        } else {
            state.motor_a_dir.set_low().unwrap();
            println!("setting motor a direction low");
        }
        
        state.motor_a_pwm.set_duty(power).unwrap();
        println!("setting motor a power: {:?}", power);
    }

    pub fn set_motor_b_speed(speed: i8) {
        let mut state = HARDWARE_STATE.get().expect("Hardware not initialized").lock().unwrap();
        
        let (power, reverse) = Self::convert(speed, state.motor_b_max_duty);
        if reverse {
            state.motor_b_dir.set_high().unwrap();
            println!("setting motor b direction high");
        } else {
            state.motor_b_dir.set_low().unwrap();
            println!("setting motor b direction low");
        }
        
        state.motor_b_pwm.set_duty(power).unwrap();
        println!("setting motor b power: {:?}", power);
    }
}