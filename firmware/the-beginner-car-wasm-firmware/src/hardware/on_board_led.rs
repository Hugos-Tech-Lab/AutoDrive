use std::{
    borrow::{Borrow, BorrowMut},
    sync::{Arc, OnceLock},
};

use esp_idf_svc::hal::{
    gpio::{AnyIOPin, Gpio8, Gpio15, Gpio21, Gpio22, Gpio23, PinDriver}, ledc::{CHANNEL0, CHANNEL1, LedcDriver, LedcTimerDriver, TIMER0, TIMER1, config::TimerConfig}, spi::{self, Dma, SpiBusDriver, SpiConfig, SpiDriver, SpiDriverConfig}, task::queue::Queue, units::Hertz,
};

use esp_idf_svc::hal::units::*;
use smart_leds_trait::{RGB8, SmartLedsWrite};
use ws2812_spi::{Ws2812, devices};

struct SpiDriverHolder {
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

#[derive(Clone, Copy, Debug)]
pub enum HardwareCommand {
    SetColor(RGB8),
    SetMotorASpeed(i8),
    SetMotorBSpeed(i8),
    Off,
}

static LED_QUEUE: OnceLock<Arc<Queue<HardwareCommand>>> = OnceLock::new();

pub struct OnBoardLed;

impl OnBoardLed {
    pub fn new(
        pin_8: Gpio8<'static>,
        spi_2: spi::SPI2<'static>,
        pin_15: Gpio15<'static>,
        pin_21: Gpio21<'static>,
        pin_22: Gpio22<'static>,
        pin_23: Gpio23<'static>,
        timer0: TIMER0<'static>,
        timer1: TIMER1<'static>,
        channel0: CHANNEL0<'static>,
        channel1: CHANNEL1<'static>,
    ) -> Self {
        // The queue is the only thing shared between tasks.
        let queue = Arc::new(Queue::new(8));

        if LED_QUEUE.set(Arc::clone(&queue)).is_err() {
            panic!("LED queue already initialized");
        }

        let mut motor_a_dir_pin = PinDriver::output(pin_15).unwrap();

        
        let motor_a_timer_driver =
            LedcTimerDriver::new(timer0, &TimerConfig::default().frequency(20.kHz().into()))
                .unwrap();

        let mut motor_a_pwm_driver =
            LedcDriver::new(channel0, motor_a_timer_driver, pin_23).unwrap();

        let mut motor_b_dir_pin = PinDriver::output(pin_22).unwrap();

        
        let motor_b_timer_driver =
            LedcTimerDriver::new(timer1, &TimerConfig::default().frequency(20.kHz().into()))
                .unwrap();

        let mut motor_b_pwm_driver =
            LedcDriver::new(channel1, motor_b_timer_driver, pin_21).unwrap();

        motor_a_dir_pin.set_low().unwrap();
        motor_b_dir_pin.set_low().unwrap();

        // Start stopped.


        let motor_a_max_duty = motor_a_pwm_driver.get_max_duty();
        let motor_b_max_duty = motor_b_pwm_driver.get_max_duty();

        motor_a_pwm_driver.set_duty(0).unwrap();
        motor_b_pwm_driver.set_duty(0).unwrap();

        // Give the NEXT pthread these settings.
        esp_idf_svc::hal::task::thread::ThreadSpawnConfiguration {
            name: Some(c"hardware"),
            stack_size: 4096,
            priority: 5,
            ..Default::default()
        }
        .set()
        .unwrap();

        std::thread::Builder::new()
            .spawn(move || {
                let bus_config = SpiDriverConfig::new().dma(Dma::Auto(4096));

                let spi_driver = SpiDriver::new_without_sclk(
                    spi_2,
                    pin_8,
                    Option::<AnyIOPin>::None,
                    &bus_config,
                )
                .unwrap();

                let spi_driver = SpiDriverHolder { spi_driver };

                let spi_config = SpiConfig::new().baudrate(Hertz(3_200_000)).write_only(true);

                let spi_bus_driver = SpiBusDriver::new(spi_driver, &spi_config).unwrap();

                let mut led = Ws2812::<_, devices::Ws2812>::new(spi_bus_driver);

                // Start with LED off.
                led.write([RGB8 { r: 0, g: 0, b: 0 }]).unwrap();

                println!("LED hardware task started");

                loop {
                    println!("LED waiting");

                    let (command, _) = queue.recv_front(u32::MAX).unwrap();

                    println!("LED command received");

                    match command {
                        HardwareCommand::SetColor(color) => {
                            led.write([color]).unwrap();
                        }

                        HardwareCommand::Off => {
                            led.write([RGB8 { r: 0, g: 0, b: 0 }]).unwrap();
                        }
                        HardwareCommand::SetMotorASpeed(speed) => {
                            let (power, reverse) = Self::convert(speed, motor_a_max_duty);
                            if reverse {
                                motor_a_dir_pin.set_high().unwrap();
                                println!("setting motor a direction high");
                            } else {
                                motor_a_dir_pin.set_low().unwrap();
                                println!("setting motor a direction low");
                            }
                            motor_a_pwm_driver.set_duty(power).unwrap();
                            println!("setting motor a power: {:?}", power);
                        }
                        HardwareCommand::SetMotorBSpeed(speed) => {
                            let (power, reverse) = Self::convert(speed, motor_b_max_duty);
                            if reverse {
                                motor_b_dir_pin.set_high().unwrap();
                                println!("setting motor b direction high");
                            } else {
                                motor_b_dir_pin.set_low().unwrap();
                                println!("setting motor b direction low");
                            }
                            motor_b_pwm_driver.set_duty(power).unwrap();
                            println!("setting motor b power: {:?}", power);
                        }
                    }
                }
            })
            .unwrap(); // TODO: Don't forget about the return 

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
        let queue = LED_QUEUE.get().expect("OnBoardLed not initialized");

        println!("sending RGB8");

        queue
            .send_back(HardwareCommand::SetColor(color), 1000)
            .unwrap();
    }

    pub fn off() {
        let queue = LED_QUEUE.get().expect("OnBoardLed not initialized");

        queue.send_back(HardwareCommand::Off, 1000).unwrap();
    }

    pub fn set_motor_a_speed(speed: i8) {
        let queue = LED_QUEUE.get().expect("OnBoardLed not initialized");

        queue.send_back(HardwareCommand::SetMotorASpeed(speed), 1000).unwrap();
    }

    pub fn set_motor_b_speed(speed: i8) {
        let queue = LED_QUEUE.get().expect("OnBoardLed not initialized");

        queue.send_back(HardwareCommand::SetMotorBSpeed(speed), 1000).unwrap();
    }
}
