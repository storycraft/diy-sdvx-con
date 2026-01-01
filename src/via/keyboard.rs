use embassy_time::Instant;

use crate::via::{ViaCmd, ViaCmdId};

/// Via keyboard value id from
/// https://github.com/qmk/qmk_firmware/blob/acbeec29dab5331fe914f35a53d6b43325881e4d/quantum/via.h#L79
struct ViaKeyboardValueId;
impl ViaKeyboardValueId {
    pub const UPTIME: u8 = 0x01;
    pub const LAYOUT_OPTIONS: u8 = 0x02;
    pub const SWITCH_MATRIX_STATE: u8 = 0x03;
    pub const FIRMWARE_VERSION: u8 = 0x04;
    pub const DEVICE_INDICATION: u8 = 0x05;
}

/// Value extracted from
/// https://github.com/qmk/qmk_firmware/blob/acbeec29dab5331fe914f35a53d6b43325881e4d/quantum/via.h#L51
const VIA_FIRMWARE_VERSION: u32 = 0x00000000;

impl ViaCmd<'_> {
    pub fn read_via_keyboard_value(mut self) {
        let value_id = self.data[0];

        match value_id {
            ViaKeyboardValueId::UPTIME => {
                let now = Instant::now().as_millis() as u32;
                self.data[1..5].copy_from_slice(&now.to_be_bytes());
            }

            ViaKeyboardValueId::FIRMWARE_VERSION => {
                self.data[1..5].copy_from_slice(&VIA_FIRMWARE_VERSION.to_be_bytes());
            }

            _ => {
                self.set_invalid();
                defmt::warn!("Invalid via keyboard value requested: {=u8:04X}", value_id);
            }
        }
    }
}
