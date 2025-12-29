mod custom;
mod keyboard;

use embassy_usb::{
    class::hid::{self, HidReaderWriter},
    driver::Driver,
};

use crate::{
    config,
    via::{
        custom::{read_custom_get_value, read_custom_save, read_custom_set_value},
        keyboard::read_via_keyboard_value,
    },
};

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
    pub const DYNAMIC_KEYMAP_GET_KEYCODE: u8 = 0x04;
    pub const DYNAMIC_KEYMAP_SET_KEYCODE: u8 = 0x05;
    pub const DYNAMIC_KEYMAP_RESET: u8 = 0x06;
    pub const CUSTOM_SET_VALUE: u8 = 0x07;
    pub const CUSTOM_GET_VALUE: u8 = 0x08;
    pub const CUSTOM_SAVE: u8 = 0x09;
    pub const DYNAMIC_KEYMAP_MACRO_GET_COUNT: u8 = 0x0C;
    pub const DYNAMIC_KEYMAP_MACRO_GET_BUFFER_SIZE: u8 = 0x0D;
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
            let ver = VIA_PROTOCOL_VERSION.to_be_bytes();
            data[1] = ver[0];
            data[2] = ver[1];
        }

        ViaCmdId::GET_KEYBOARD_VALUE => {
            read_via_keyboard_value(data).await;
        }

        ViaCmdId::DYNAMIC_KEYMAP_GET_KEYCODE => {
            let _layer = data[1];
            let _row = data[2];
            let _col = data[3];

            let key = 0_u16.to_be_bytes();
            data[4] = key[0];
            data[5] = key[1];
        }

        ViaCmdId::DYNAMIC_KEYMAP_SET_KEYCODE => {
            let _layer = data[1];
            let _row = data[2];
            let _col = data[3];
            let _key = ((data[4] as u16) << 8) | data[5] as u16;
            // TODO
        }

        ViaCmdId::CUSTOM_GET_VALUE => {
            read_custom_get_value(data).await;
        }

        ViaCmdId::CUSTOM_SET_VALUE => {
            read_custom_set_value(data).await;
        }

        ViaCmdId::CUSTOM_SAVE => {
            read_custom_save(data).await;
        }

        // Disable Macro
        ViaCmdId::DYNAMIC_KEYMAP_MACRO_GET_COUNT => {
            data[1] = 0;
        }

        // Disable Macro
        ViaCmdId::DYNAMIC_KEYMAP_MACRO_GET_BUFFER_SIZE => {
            data[1] = 0;
        }

        ViaCmdId::DYNAMIC_KEYMAP_GET_LAYER_COUNT => {
            // Hardcode layer count 1
            data[1] = 1;
        }

        ViaCmdId::DYNAMIC_KEYMAP_GET_BUFFER => {
            let _layer = data[1];
            let _row = data[2];
            let _col = data[3];
        }

        ViaCmdId::DYNAMIC_KEYMAP_GET_ENCODER => {
            let _layer = data[1];
            let _encoder_id = data[2];
            let _clockwise = data[3] != 0;
            let _keycode = (data[4] as u16) << 8 | data[5] as u16;
        }

        ViaCmdId::DYNAMIC_KEYMAP_SET_ENCODER => {
            // TODO
            let _layer = data[1];
            let _encoder_id = data[2];
            let _clockwise = data[3] != 0;
            let _keycode = (data[4] as u16) << 8 | data[5] as u16;
        }

        _ => {
            log::warn!("Invalid via command recevied: {id:#04X}");
            data[0] = ViaCmdId::UNHANDLED;
        }
    }
}
