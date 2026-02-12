use embassy_rp::rom_data;

use crate::{
    userdata::{self},
    via::ViaCmd,
};

struct ValueId;
impl ValueId {
    /// Reboot to BOOTSEL
    pub const REBOOT_BOOTSEL: u8 = 0x02;
    /// SDVX EAC Mode
    pub const EAC_MODE: u8 = 0x03;
}

impl ViaCmd<'_> {
    pub fn read_custom_get_value(self) {
        let channel_id = self.data[0];

        // 0 for custom user defined channel
        // We will only use user defined channel so ignore rest
        if channel_id != 0 {
            self.set_invalid();
            return;
        }

        let value_id = self.data[1];
        match value_id {
            ValueId::REBOOT_BOOTSEL | ValueId::EAC_MODE => {
                // Fixed value
                self.data[2] = 1;
            }

            _ => {
                self.set_invalid();
            }
        }
    }

    pub fn read_custom_set_value(self) {
        let channel_id = self.data[0];

        // 0 for custom user defined channel
        // We will only use user defined channel so ignore rest
        if channel_id != 0 {
            self.set_invalid();
            return;
        }

        let value_id = self.data[1];
        match value_id {
            ValueId::REBOOT_BOOTSEL => {
                defmt::info!("BOOTSEL Reboot requested.");
                // Reboot to BOOTSEL
                rom_data::reset_to_usb_boot(0, 0);
            }

            ValueId::EAC_MODE => {
                defmt::info!("EAC Mode enabled.");
                userdata::update(|data| {
                    data.eac_mode = true;
                });
                userdata::save();

                rom_data::reboot(0, 1, 0, 0);
            }

            _ => {
                self.set_invalid();
            }
        }
    }

    pub fn read_custom_save(self) {
        let channel_id = self.data[0];

        // 0 for custom user defined channel
        // We will only use user defined channel so ignore rest
        if channel_id != 0 {
            self.set_invalid();
            return;
        }

        userdata::save();
    }
}
