mod cmds;
mod custom;
mod encoder;
mod keyboard;
mod keymap;

use embassy_executor::SpawnToken;
use embassy_usb::class::hid::{HidReaderWriter, State};
use num_traits::FromPrimitive;
use static_cell::StaticCell;
use zerocopy::{FromBytes, IntoBytes, big_endian};

use crate::{
    keycode::Keycode,
    usb::{self, Driver, hid::QmkRawHidReport},
    userdata::{self, keymap::Keymap},
    via::{
        cmds::*,
        encoder::{get_encoder_keycode, set_encoder_keycode},
        keymap::{KeymapBuffer, get_keymap_keycode, set_keymap_keycode},
    },
};

pub fn via_task(
    builder: &mut embassy_usb::Builder<'static, Driver>,
) -> SpawnToken<impl Sized + use<>> {
    #[embassy_executor::task]
    async fn inner(mut io: HidReaderWriter<'static, Driver, 32, 32>) {
        io.ready().await;
        let (mut reader, mut writer) = io.split();

        let mut buf = [0_u8; { size_of::<QmkRawHidReport>() }];
        loop {
            if let Err(e) = reader.read(&mut buf).await {
                defmt::error!("Failed to send via report err:{:?}", e);
                continue;
            }

            let Some(cmd) = ViaCmd::from_raw(&mut buf) else {
                defmt::error!("Failed parse via report");
                continue;
            };

            cmd.invoke().await;

            if let Err(err) = writer.write(&buf).await {
                defmt::error!("Failed to write via report. err: {:?}", err);
            }
        }
    }

    let io = HidReaderWriter::<_, { size_of::<QmkRawHidReport>() }, { size_of::<QmkRawHidReport>() }>::new(
        builder,
        {
            static STATE: StaticCell<State> = StaticCell::new();
            STATE.init(State::new())
        },
        usb::config::via(),
    );
    inner(io)
}

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

struct ViaCmd<'a> {
    id: &'a mut u8,
    data: &'a mut [u8],
}

impl<'a> ViaCmd<'a> {
    #[inline]
    fn from_raw(raw: &'a mut [u8]) -> Option<Self> {
        let (id, data) = raw.split_first_mut().unwrap();
        Some(Self { id, data })
    }

    #[inline]
    fn set_invalid(self) {
        *self.id = ViaCmdId::UNHANDLED;
    }

    async fn invoke(self) {
        match *self.id {
            ViaCmdId::GET_PROTOCOL_VERSION => {
                GetProtocolVersion::mut_from_prefix(self.data)
                    .unwrap()
                    .0
                    .version = GetProtocolVersion::CURRENT_VERSION.into();
                defmt::info!("Via connected.");
            }

            ViaCmdId::GET_KEYBOARD_VALUE => {
                self.read_via_keyboard_value();
            }

            ViaCmdId::DYNAMIC_KEYMAP_GET_KEYCODE => {
                let cmd = DynamicKeymapKeycode::mut_from_prefix(self.data).unwrap().0;

                let key = userdata::get(|userdata| {
                    get_keymap_keycode(&userdata.keymap, cmd.row, cmd.col)
                })
                .unwrap_or_default();
                cmd.key = big_endian::U16::new(key as u16);
            }

            ViaCmdId::DYNAMIC_KEYMAP_SET_KEYCODE => {
                let cmd = DynamicKeymapKeycode::mut_from_prefix(self.data).unwrap().0;
                let key = Keycode::from_u16(cmd.key.get()).unwrap_or_default();

                userdata::update(|userdata| {
                    set_keymap_keycode(&mut userdata.keymap, cmd.row, cmd.col, key);
                });
                userdata::save();
                defmt::info!(
                    "Keycode at row: {} col: {} updated to key: {:#06X}",
                    cmd.row,
                    cmd.col,
                    key as u16
                );
            }

            ViaCmdId::DYNAMIC_KEYMAP_RESET => {
                userdata::update(|userdata| {
                    userdata.keymap = Keymap::DEFAULT;
                });
                defmt::info!("Keymap resetted to default.");
            }

            ViaCmdId::CUSTOM_GET_VALUE => {
                self.read_custom_get_value();
            }

            ViaCmdId::CUSTOM_SET_VALUE => {
                self.read_custom_set_value();
            }

            ViaCmdId::CUSTOM_SAVE => {
                self.read_custom_save();
            }

            // Disable Macro
            ViaCmdId::DYNAMIC_KEYMAP_MACRO_GET_COUNT => {
                self.data[0] = 0;
            }

            // Disable Macro
            ViaCmdId::DYNAMIC_KEYMAP_MACRO_GET_BUFFER_SIZE => {
                self.data[0] = 0;
            }

            ViaCmdId::DYNAMIC_KEYMAP_GET_LAYER_COUNT => {
                // Hardcode layer count 1
                self.data[0] = 1;
            }

            ViaCmdId::DYNAMIC_KEYMAP_GET_BUFFER => {
                let (cmd, buf) = DynamicKeymapBuffer::mut_from_prefix(self.data).unwrap();
                let offset = cmd.offset.get() as usize;
                let size = cmd.size as usize;

                let keymap_buf =
                    userdata::get(|userdata| KeymapBuffer::from_keymap(&userdata.keymap));

                let Some(src) = keymap_buf.as_bytes().get(offset..(offset + size)) else {
                    self.set_invalid();
                    return;
                };
                let Some(dst) = buf.get_mut(..size) else {
                    self.set_invalid();
                    return;
                };
                dst.copy_from_slice(src);
            }

            ViaCmdId::DYNAMIC_KEYMAP_SET_BUFFER => {
                let (cmd, buf) = DynamicKeymapBuffer::mut_from_prefix(self.data).unwrap();
                let offset = cmd.offset.get() as usize;
                let size = cmd.size as usize;

                let mut keymap_buf =
                    userdata::get(|userdata| KeymapBuffer::from_keymap(&userdata.keymap));

                let Some(dst) = keymap_buf.as_mut_bytes().get_mut(offset..(offset + size)) else {
                    self.set_invalid();
                    return;
                };
                let Some(src) = buf.get(..size) else {
                    self.set_invalid();
                    return;
                };
                dst.copy_from_slice(src);

                userdata::update(|userdata| keymap_buf.apply_keymap(&mut userdata.keymap));
                userdata::save();
            }

            ViaCmdId::DYNAMIC_KEYMAP_GET_ENCODER => {
                let cmd = DynamicKeymapEncoder::mut_from_prefix(self.data).unwrap().0;

                let key = get_encoder_keycode(cmd.encoder_id, cmd.clockwise != 0)
                    .unwrap_or_default() as u16;
                cmd.key = big_endian::U16::new(key);
            }

            ViaCmdId::DYNAMIC_KEYMAP_SET_ENCODER => {
                let cmd = DynamicKeymapEncoder::mut_from_prefix(self.data).unwrap().0;

                if let Some(key) = Keycode::from_u16(cmd.key.get()) {
                    set_encoder_keycode(cmd.encoder_id, cmd.clockwise != 0, key);
                    userdata::save();
                }
            }

            _ => {
                defmt::warn!("Invalid via command recevied: {:#04X}", *self.id);
                self.set_invalid();
            }
        }
    }
}
