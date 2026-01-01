use num_traits::FromPrimitive;
use zerocopy::{FromBytes, Immutable, IntoBytes, big_endian};

use crate::{keycode::Keycode, userdata::keymap::Keymap};

#[derive(PartialEq, Eq, FromBytes, IntoBytes, Immutable)]
#[repr(C)]
/// QMK dynamic keymap buffer emulation
pub struct KeymapBuffer {
    _unused0: big_endian::U16,
    pub start: big_endian::U16,
    _unused1: [big_endian::U16; 2],

    pub button1: big_endian::U16,
    pub button2: big_endian::U16,
    pub button3: big_endian::U16,
    pub button4: big_endian::U16,

    pub fx1: big_endian::U16,
    pub fx2: big_endian::U16,
    _unused2: [big_endian::U16; 2],
}

impl KeymapBuffer {
    pub fn from_keymap(keymap: &Keymap) -> Self {
        Self {
            _unused0: big_endian::U16::ZERO,
            start: big_endian::U16::new(keymap.start as _),
            _unused1: [big_endian::U16::ZERO; 2],

            button1: big_endian::U16::new(keymap.button1 as _),
            button2: big_endian::U16::new(keymap.button2 as _),
            button3: big_endian::U16::new(keymap.button3 as _),
            button4: big_endian::U16::new(keymap.button4 as _),
            fx1: big_endian::U16::new(keymap.fx1 as _),
            fx2: big_endian::U16::new(keymap.fx2 as _),
            _unused2: [big_endian::U16::ZERO; 2],
        }
    }

    pub fn apply_keymap(&self, map: &mut Keymap) {
        map.start = Keycode::from_u16(self.start.get()).unwrap_or_default();

        map.button1 = Keycode::from_u16(self.button1.get()).unwrap_or_default();
        map.button2 = Keycode::from_u16(self.button2.get()).unwrap_or_default();
        map.button3 = Keycode::from_u16(self.button3.get()).unwrap_or_default();
        map.button4 = Keycode::from_u16(self.button4.get()).unwrap_or_default();

        map.fx1 = Keycode::from_u16(self.fx1.get()).unwrap_or_default();
        map.fx2 = Keycode::from_u16(self.fx2.get()).unwrap_or_default();
    }
}

/// Get key from row, col layout
pub fn get_keymap_keycode(map: &Keymap, row: u8, col: u8) -> Option<Keycode> {
    match (row, col) {
        (0, 1) => Some(map.start),

        (1, 0) => Some(map.button1),
        (1, 1) => Some(map.button2),
        (1, 2) => Some(map.button3),
        (1, 3) => Some(map.button4),

        (2, 0) => Some(map.fx1),
        (2, 1) => Some(map.fx2),

        _ => None,
    }
}

/// Set key from row, col layout
pub fn set_keymap_keycode(map: &mut Keymap, row: u8, col: u8, code: Keycode) {
    match (row, col) {
        (0, 1) => {
            map.start = code;
        }

        (1, 0) => {
            map.button1 = code;
        }
        (1, 1) => {
            map.button2 = code;
        }
        (1, 2) => {
            map.button3 = code;
        }
        (1, 3) => {
            map.button4 = code;
        }

        (2, 0) => {
            map.fx1 = code;
        }
        (2, 1) => {
            map.fx2 = code;
        }

        _ => {}
    }
}
