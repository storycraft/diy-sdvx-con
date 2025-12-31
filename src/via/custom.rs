use embassy_rp::rom_data;

use crate::{
    userdata::{self},
    via::{ViaCmd, ViaCmdId},
};

struct ValueId;
impl ValueId {
    /// Reboot to BOOTSEL
    pub const REBOOT_BOOTSEL: u8 = 0x02;
}

impl ViaCmd<'_> {
    pub fn read_custom_get_value(self) {
        let channel_id = self.data[0];

        // 0 for custom user defined channel
        // We will only use user defined channel so ignore rest
        if channel_id != 0 {
            *self.id = ViaCmdId::UNHANDLED;
            return;
        }

        let value_id = self.data[1];
        match value_id {
            ValueId::REBOOT_BOOTSEL => {
                // Fixed value
                self.data[2] = 1;
            }

            _ => {
                *self.id = ViaCmdId::UNHANDLED;
            }
        }
    }

    pub fn read_custom_set_value(self) {
        let channel_id = self.data[0];

        // 0 for custom user defined channel
        // We will only use user defined channel so ignore rest
        if channel_id != 0 {
            *self.id = ViaCmdId::UNHANDLED;
            return;
        }

        let value_id = self.data[1];
        match value_id {
            ValueId::REBOOT_BOOTSEL => {
                defmt::info!("BOOTSEL Reboot requested.");
                // Reboot to BOOTSEL
                rom_data::reset_to_usb_boot(0, 0);
            }

            _ => {
                *self.id = ViaCmdId::UNHANDLED;
            }
        }
    }

    pub fn read_custom_save(self) {
        let channel_id = self.data[0];

        // 0 for custom user defined channel
        // We will only use user defined channel so ignore rest
        if channel_id != 0 {
            *self.id = ViaCmdId::UNHANDLED;
            return;
        }

        userdata::save();
    }
}
