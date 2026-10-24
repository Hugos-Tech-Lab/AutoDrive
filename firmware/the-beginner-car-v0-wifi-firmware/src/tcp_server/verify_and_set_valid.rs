use esp_idf_svc::ota::{EspOta, SlotState};
use log::info;

pub fn verify_and_set_valid(ota: &mut EspOta) -> anyhow::Result<()> {
    let running_slot = ota.get_running_slot()?;

    // Factory Reset slots should not be marked. Trying to mark it will not crash but will display
    // some errors in the logs.
    if running_slot.state == SlotState::Factory {
        info!("Factory slot can't be marked");
        return Ok(());
    }

    if running_slot.state != SlotState::Valid {
        let is_app_valid = true;

        if is_app_valid {
            ota.mark_running_slot_valid()?;
        } else {
            ota.mark_running_slot_invalid_and_reboot();
        }
    }

    Ok(())
}