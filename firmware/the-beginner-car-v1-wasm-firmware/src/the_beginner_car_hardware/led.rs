use esp_idf_svc::hal::gpio::Output;
use esp_idf_svc::hal::gpio::{AnyIOPin, PinDriver};

pub struct Led {
    pin: PinDriver<'static, Output>,
}

impl Led {
    pub fn new(pin: AnyIOPin<'static>) -> Self {
        let pin = PinDriver::output(pin).unwrap();

        Self { pin }
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.off()?;
        Ok(())
    }

    pub fn on(&mut self) -> anyhow::Result<()> {
        self.pin.set_high().unwrap();
        Ok(())
    }

    pub fn off(&mut self) -> anyhow::Result<()> {
        self.pin.set_low().unwrap();
        Ok(())
    }
}

impl Drop for Led {
    fn drop(&mut self) {
        if let Err(err) = self.stop() {
            log::error!("Could not stop motor on drop: '{err}'");
        }
    }
}
