use embassy_time::Instant;
use embassy_usb::{
    class::hid::{self, HidReaderWriter},
    driver::Driver,
};

use crate::config;

pub fn via_task<'a, D: Driver<'a>>(
    state: &'a mut hid::State<'a>,
    builder: &mut embassy_usb::Builder<'a, D>,
) -> impl Future<Output = ()> + use<'a, D> {
    let mut io = HidReaderWriter::<_, 32, 32>::new(builder, state, config::usb_via_config());
    async move {
        io.ready().await;
        let (mut reader, mut writer) = io.split();

        let mut buf = [0_u8; 32];
        loop {
            if let Err(e) = reader.read(&mut buf).await {
                log::error!("Failed to send via report: {:?}", e);
                continue;
            }

            read_via_cmd(&mut buf).await;
            if let Err(err) = writer.write(&buf).await {
                log::error!("Failed to response via report. {:?}", err);
            }
        }
    }
}

/// Via protocol version from
/// https://github.com/qmk/qmk_firmware/blob/acbeec29dab5331fe914f35a53d6b43325881e4d/quantum/via.h#L42
const VIA_PROTOCOL_VERSION: u16 = 0x000C;

/// Via command ids from
/// https://github.com/qmk/qmk_firmware/blob/acbeec29dab5331fe914f35a53d6b43325881e4d/quantum/via.h#L54
struct ViaCmdId;
impl ViaCmdId {
    pub const GET_PROTOCOL_VERSION: u8 = 0x01;
    pub const GET_KEYBOARD_VALUE: u8 = 0x02;
    pub const DYNAMIC_KEYMAP_GET_LAYER_COUNT: u8 = 0x11;
    pub const DYNAMIC_KEYMAP_GET_BUFFER: u8 = 0x12;
    pub const DYNAMIC_KEYMAP_SET_BUFFER: u8 = 0x13;
    pub const DYNAMIC_KEYMAP_GET_ENCODER: u8 = 0x14;
    pub const DYNAMIC_KEYMAP_SET_ENCODER: u8 = 0x15;
    pub const UNHANDLED: u8 = 0xff;
}

async fn read_via_cmd(data: &mut [u8]) {
    let id = data[0];
    match id {
        ViaCmdId::GET_PROTOCOL_VERSION => {
            let ver = VIA_PROTOCOL_VERSION.to_ne_bytes();
            data[1] = ver[0];
            data[2] = ver[1];
        }

        ViaCmdId::GET_KEYBOARD_VALUE => {
            read_via_keyboard_value(&mut data[1..]).await;
        }

        ViaCmdId::DYNAMIC_KEYMAP_GET_LAYER_COUNT => {
            // Hardcode layer count 1
            data[1] = 1;
        }

        ViaCmdId::DYNAMIC_KEYMAP_GET_BUFFER => {}

        ViaCmdId::DYNAMIC_KEYMAP_GET_ENCODER => {}

        ViaCmdId::DYNAMIC_KEYMAP_SET_ENCODER => {}

        _ => {
            log::warn!("Invalid via command recevied: {id:#04X}");
            data[0] = ViaCmdId::UNHANDLED;
        }
    }
}

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

async fn read_via_keyboard_value<'a>(data: &mut [u8]) {
    let id = data[0];

    match id {
        ViaKeyboardValueId::UPTIME => {
            let now = (Instant::now().as_millis() as u32).to_be_bytes();
            data[1] = now[0];
            data[2] = now[1];
            data[3] = now[2];
            data[4] = now[3];
        }

        _ => {
            log::warn!("Invalid via keyboard value requested: {id:#04X}");
        }
    }
}
