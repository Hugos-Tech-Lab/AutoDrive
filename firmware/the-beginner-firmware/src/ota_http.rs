use std::sync::{Arc, Mutex};

use esp_idf_svc::{http::server::EspHttpServer, ota::EspOta};

pub mod update;
pub mod info;
pub mod logs;
pub mod verify_and_set_valid;
pub mod autoscript_manager;

pub fn set_handles(mut server: &mut EspHttpServer, esp_ota: Arc<Mutex<EspOta>>) -> anyhow::Result<()> {
    update::set_handles(&mut server, esp_ota.clone())?;
    info::set_handles(&mut server, esp_ota.clone())?;
    logs::set_handles(&mut server)?;

    Ok(())
}
