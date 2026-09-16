use std::{
    borrow::{Borrow, BorrowMut},
    sync::{Arc, OnceLock},
};

use esp_idf_svc::hal::{
    gpio::{AnyIOPin, Gpio8},
    spi::{self, Dma, SpiBusDriver, SpiConfig, SpiDriver, SpiDriverConfig},
    task::queue::Queue,
    units::Hertz,
};

use smart_leds_trait::{RGB8, SmartLedsWrite};
use ws2812_spi::{devices, Ws2812};

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
pub enum LedCommand {
    SetColor(RGB8),
    Off,
}

static LED_QUEUE: OnceLock<Arc<Queue<LedCommand>>> = OnceLock::new();

pub struct OnBoardLed;

impl OnBoardLed {
    pub fn new(
        pin_8: Gpio8<'static>,
        spi_2: spi::SPI2<'static>,
    ) -> Self {
        // The queue is the only thing shared between tasks.
        let queue = Arc::new(Queue::new(8));

        if LED_QUEUE.set(Arc::clone(&queue)).is_err() {
            panic!("LED queue already initialized");
        }

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
                let bus_config =
                    SpiDriverConfig::new()
                        .dma(Dma::Auto(4096));

                let spi_driver =
                    SpiDriver::new_without_sclk(
                        spi_2,
                        pin_8,
                        Option::<AnyIOPin>::None,
                        &bus_config,
                    )
                    .unwrap();

                let spi_driver =
                    SpiDriverHolder { spi_driver };

                let spi_config =
                    SpiConfig::new()
                        .baudrate(Hertz(3_200_000))
                        .write_only(true);

                let spi_bus_driver =
                    SpiBusDriver::new(
                        spi_driver,
                        &spi_config,
                    )
                    .unwrap();

                let mut led =
                    Ws2812::<_, devices::Ws2812>::new(
                        spi_bus_driver,
                    );

                // Start with LED off.
                led.write([RGB8 {
                    r: 0,
                    g: 0,
                    b: 0,
                }])
                .unwrap();

                println!("LED hardware task started");

                loop {
                    println!("LED waiting");

                    let (command, _) =
                        queue.recv_front(u32::MAX).unwrap();

                    println!("LED command received");

                    match command {
                        LedCommand::SetColor(color) => {
                            led.write([color]).unwrap();
                        }

                        LedCommand::Off => {
                            led.write([RGB8 {
                                r: 0,
                                g: 0,
                                b: 0,
                            }])
                            .unwrap();
                        }
                    }
                }
            })
            .unwrap();

        Self
    }

    pub fn set_color(color: RGB8) {
        let queue = LED_QUEUE
            .get()
            .expect("OnBoardLed not initialized");

        println!("sending RGB8");

        queue
            .send_back(
                LedCommand::SetColor(color),
                1000,
            )
            .unwrap();
    }

    pub fn off() {
        let queue = LED_QUEUE
            .get()
            .expect("OnBoardLed not initialized");

        queue
            .send_back(LedCommand::Off, 1000)
            .unwrap();
    }
}
