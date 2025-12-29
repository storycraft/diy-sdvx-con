use embassy_rp::rom_data;

use crate::{
    userdata::{self, InputMode},
    via::ViaCmdId,
};

struct ValueId;
impl ValueId {
    /// Set controller input mode
    pub const CONTROLLER_MODE: u8 = 0x01;
    /// Reboot to BOOTSEL
    pub const REBOOT_BOOTSEL: u8 = 0x02;
}

pub async fn read_custom_get_value(data: &mut [u8]) {
    let channel_id = data[1];

    // 0 for custom user defined channel
    // We will only use user defined channel so ignore rest
    if channel_id != 0 {
        data[0] = ViaCmdId::UNHANDLED;
        return;
    }

    let value_id = data[2];
    match value_id {
        ValueId::CONTROLLER_MODE => {
            userdata::update(|userdata| {
                data[3] = userdata.input_mode.to_num();
            });
        }

        ValueId::REBOOT_BOOTSEL => {
            // Fixed value
            data[3] = 1;
        }

        _ => {
            data[0] = ViaCmdId::UNHANDLED;
        }
    }
}

pub async fn read_custom_set_value(data: &mut [u8]) {
    let channel_id = data[1];

    // 0 for custom user defined channel
    // We will only use user defined channel so ignore rest
    if channel_id != 0 {
        data[0] = ViaCmdId::UNHANDLED;
        return;
    }

    let value_id = data[2];
    match value_id {
        ValueId::CONTROLLER_MODE => {
            let Some(mode) = InputMode::from_num(data[3]) else {
                log::info!("Invalid InputMode mode: {}", data[3]);
                data[0] = ViaCmdId::UNHANDLED;
                return;
            };

            userdata::update(|userdata| {
                userdata.input_mode = mode;
            });
            log::info!("Controller mode updated.");
        }

        ValueId::REBOOT_BOOTSEL => {
            log::info!("BOOTSEL Reboot requested.");
            // Reboot to BOOTSEL
            rom_data::reset_to_usb_boot(0, 0);
        }

        _ => {
            data[0] = ViaCmdId::UNHANDLED;
        }
    }
}

pub async fn read_custom_save(data: &mut [u8]) {
    let channel_id = data[1];

    // 0 for custom user defined channel
    // We will only use user defined channel so ignore rest
    if channel_id != 0 {
        data[0] = ViaCmdId::UNHANDLED;
    }

    userdata::save();
}
