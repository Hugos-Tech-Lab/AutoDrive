use std::{
    borrow::{Borrow, BorrowMut},
};

use esp_idf_svc::hal::{
    gpio::{AnyIOPin},
    ledc::{CHANNEL0, CHANNEL1, LedcDriver, LedcTimerDriver, TIMER0, TIMER1, config::TimerConfig},
    spi::{self, Dma, SpiBusDriver, SpiConfig, SpiDriver, SpiDriverConfig},
    units::Hertz,
};
use smart_leds_trait::{RGB8, SmartLedsWrite};
use ws2812_spi::{Ws2812, devices};

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
pub struct HardwareState {
    led: Ws2812<SpiBusDriver<'static, SpiDriverHolder>, devices::Ws2812>,
}

// yeee hawwhh
unsafe impl Send for HardwareState {}

pub struct OnBoardLED {
    hardware: HardwareState
}

impl OnBoardLED {
    pub fn new(rgb_pin: AnyIOPin<'static>, spi_2: spi::SPI2<'static>) -> Self {
        // --- Setup LED ---
        let bus_config = SpiDriverConfig::new().dma(Dma::Auto(4096));
        let spi_driver =
            SpiDriver::new_without_sclk(spi_2, rgb_pin, Option::<AnyIOPin>::None, &bus_config)
                .unwrap();

        let spi_driver = SpiDriverHolder { spi_driver };
        let spi_config = SpiConfig::new().baudrate(Hertz(3_200_000)).write_only(true);
        let spi_bus_driver = SpiBusDriver::new(spi_driver, &spi_config).unwrap();

        let mut led = Ws2812::<_, devices::Ws2812>::new(spi_bus_driver);

        // Start with LED off
        led.write([RGB8 { r: 0, g: 0, b: 0 }]).unwrap();

        // Construct the state payload
        let hardware = HardwareState { led };

        Self {
            hardware
        }
    }

    fn convert(value: i8, max: u32) -> (u32, bool) {
        let negative = value < 0;
        let magnitude = value.unsigned_abs() as u32;

        // Example mapping: -128..=127 → 0..=max
        let output = magnitude * max / 128;

        (output, negative)
    }

    pub fn set_color(&mut self, r: u8, g: u8, b: u8) {
        self.hardware.led.write([RGB8{ r, g, b }]).unwrap();
    }

    pub fn off(&mut self) {
        self.hardware.led.write([RGB8 { r: 0, g: 0, b: 0 }]).unwrap();
    }
}

