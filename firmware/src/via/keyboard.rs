use embassy_rp::gpio::Level;
use embassy_time::Instant;

use crate::{input::CURRENT_INPUT, via::ViaCmd};

/// Via keyboard value id from
/// https://github.com/qmk/qmk_firmware/blob/acbeec29dab5331fe914f35a53d6b43325881e4d/quantum/via.h#L79
struct ViaKeyboardValueId;
impl ViaKeyboardValueId {
    pub const UPTIME: u8 = 0x01;
    pub const SWITCH_MATRIX_STATE: u8 = 0x03;
    pub const FIRMWARE_VERSION: u8 = 0x04;
    pub const DEVICE_INDICATION: u8 = 0x05;
}

/// Value extracted from
/// https://github.com/qmk/qmk_firmware/blob/acbeec29dab5331fe914f35a53d6b43325881e4d/quantum/via.h#L51
const VIA_FIRMWARE_VERSION: u32 = 0x00000000;

impl ViaCmd<'_> {
    pub fn read_via_keyboard_value(self) {
        let value_id = self.data[0];

        match value_id {
            ViaKeyboardValueId::UPTIME => {
                let now = Instant::now().as_millis() as u32;
                self.data[1..5].copy_from_slice(&now.to_be_bytes());
            }

            ViaKeyboardValueId::FIRMWARE_VERSION => {
                self.data[1..5].copy_from_slice(&VIA_FIRMWARE_VERSION.to_be_bytes());
            }

            ViaKeyboardValueId::SWITCH_MATRIX_STATE => {
                let offset = self.data[1];

                let read = CURRENT_INPUT.borrow().get();
                let matrix = [
                    ((read.buttons.start == Level::High) as u8) << 1,
                    ((read.buttons.button1 == Level::High) as u8)
                        | ((read.buttons.button2 == Level::High) as u8) << 1
                        | ((read.buttons.button3 == Level::High) as u8) << 2
                        | ((read.buttons.button4 == Level::High) as u8) << 3,
                    ((read.buttons.fx1 == Level::High) as u8)
                        | ((read.buttons.fx2 == Level::High) as u8) << 1,
                ];

                if let Some(matrix_slice) = matrix.get((offset as usize)..) {
                    self.data[2..][..matrix_slice.len()].copy_from_slice(matrix_slice);
                }
            }

            ViaKeyboardValueId::DEVICE_INDICATION => {
                let value = self.data[1];
                if value != 0 {
                    return;
                }

                defmt::info!("Via device ACK");
            }

            _ => {
                self.set_invalid();
                defmt::warn!("Invalid via keyboard value requested: {=u8:04X}", value_id);
            }
        }
    }
}
