use std::{
    borrow::Borrow,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU16, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

use esp_idf_svc::hal::{
    adc::{
        ADC1, AdcChannel,
        attenuation::DB_12,
        oneshot::{AdcChannelDriver, AdcDriver, config::AdcChannelConfig},
    },
    delay::FreeRtos,
    gpio::{Gpio0, Gpio1, Gpio2, Gpio3, Gpio4, Gpio5, Gpio6},
};
use the_beginner_car_spi_protocol::Event;

pub struct IrSensorArrayHardware {
    pub adc: ADC1<'static>,
    pub front_far_left: Gpio0<'static>,
    pub front_left: Gpio1<'static>,
    pub front_center: Gpio2<'static>,
    pub front_right: Gpio3<'static>,
    pub front_far_right: Gpio4<'static>,
    pub back_center: Gpio5<'static>,
    pub back_left: Gpio6<'static>,
}

#[derive(Default)]
pub struct IrSensorArrayData {
    // front
    pub front_far_left: AtomicU16,
    pub front_left: AtomicU16,
    pub front_center: AtomicU16,
    pub front_right: AtomicU16,
    pub front_far_right: AtomicU16,
    // back
    pub back_center: AtomicU16,
    pub back_left: AtomicU16,
}

pub struct IrSensorArray {}

impl IrSensorArray {
    pub fn new<T: AdcChannel>(
        hardware: IrSensorArrayHardware,
        events: Arc<Mutex<Vec<Event>>>,
        data: Arc<IrSensorArrayData>,
    ) -> Self {
        let a = Self::long_lived_watch_readings(hardware, events, data);

        Self {}
    }

    fn read_and_store<'d, C, M>(pin: &mut AdcChannelDriver<'d, C, M>, atomic: &AtomicU16) -> u16
    where
        C: AdcChannel,
        M: Borrow<AdcDriver<'d, C::AdcUnit>>,
    {
        let reading = ((pin.read_raw().unwrap() as u32 * u16::MAX as u32) / 4095) as u16;
        atomic.store(reading, Ordering::Relaxed);
        reading
    }

    pub fn long_lived_watch_readings(
        hardware: IrSensorArrayHardware,
        events: Arc<Mutex<Vec<Event>>>,
        data: Arc<IrSensorArrayData>,
    ) -> JoinHandle<!> {
        let reading = std::thread::Builder::new()
            .stack_size(2_000)
            .name("sensor_readings".to_string())
            .spawn({
                move || {
                    let adc = AdcDriver::new(hardware.adc).unwrap();
                    let config = AdcChannelConfig {
                        attenuation: DB_12,
                        ..Default::default()
                    };

                    let mut front_far_left =
                        AdcChannelDriver::new(&adc, hardware.front_far_left, &config).unwrap();
                    let mut front_left =
                        AdcChannelDriver::new(&adc, hardware.front_left, &config).unwrap();
                    let mut front_center =
                        AdcChannelDriver::new(&adc, hardware.front_center, &config).unwrap();
                    let mut front_right =
                        AdcChannelDriver::new(&adc, hardware.front_right, &config).unwrap();
                    let mut front_far_right =
                        AdcChannelDriver::new(&adc, hardware.front_far_right, &config).unwrap();
                    let mut back_center =
                        AdcChannelDriver::new(&adc, hardware.back_center, &config).unwrap();
                    let mut back_left =
                        AdcChannelDriver::new(&adc, hardware.back_left, &config).unwrap();

                    let mut last_flush = std::time::Instant::now();

                    loop {
                        // TODO: this polling could be based on the speed of the wheels
                        let front_far_left =
                            Self::read_and_store(&mut front_far_left, &data.front_far_left);
                        let front_left = Self::read_and_store(&mut front_left, &data.front_left);
                        let front_center =
                            Self::read_and_store(&mut front_center, &data.front_center);
                        let front_right = Self::read_and_store(&mut front_right, &data.front_right);
                        let front_far_right =
                            Self::read_and_store(&mut front_far_right, &data.front_far_right);

                        let back_left = Self::read_and_store(&mut back_left, &data.back_left);
                        let back_center = Self::read_and_store(&mut back_center, &data.back_center);

                        if last_flush.elapsed() >= Duration::from_millis(100) {
                            let mut events = events.lock().unwrap();

                            events.push(Event::InfraredLedReadings {
                                front_far_left,
                                front_left,
                                front_center,
                                front_right,
                                front_far_right,
                                back_left,
                                back_center,
                            });

                            last_flush = std::time::Instant::now();
                        }

                        FreeRtos::delay_ms(1);
                    }
                }
            })
            .unwrap();

        reading
    }
}
