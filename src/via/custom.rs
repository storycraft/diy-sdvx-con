use embassy_rp::rom_data;

use crate::via::ViaCmdId;

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
            // TODO:: 0 Gamepad, 1 Keyboard + Mouse
            data[3] = 0;
        }

        ValueId::REBOOT_BOOTSEL => {
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
            // TODO
        }

        ValueId::REBOOT_BOOTSEL => {
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
        return;
    }
}
