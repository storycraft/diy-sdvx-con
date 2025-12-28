use embassy_time::Instant;

use crate::via::ViaCmdId;

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

pub async fn read_via_keyboard_value<'a>(data: &mut [u8]) {
    let id = data[1];

    match id {
        ViaKeyboardValueId::UPTIME => {
            let now = (Instant::now().as_millis() as u32).to_be_bytes();
            data[2] = now[0];
            data[3] = now[1];
            data[4] = now[2];
            data[5] = now[3];
        }

        _ => {
            data[0] = ViaCmdId::UNHANDLED;
            log::warn!("Invalid via keyboard value requested: {id:#04X}");
        }
    }
}
