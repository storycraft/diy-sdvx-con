use keycode::Keycode;
use zerocopy::{FromBytes, Immutable, IntoBytes, big_endian};

use crate::userdata::keymap::Keymap;

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
            start: big_endian::U16::new(keymap.start.0),
            _unused1: [big_endian::U16::ZERO; 2],

            button1: big_endian::U16::new(keymap.button1.0),
            button2: big_endian::U16::new(keymap.button2.0),
            button3: big_endian::U16::new(keymap.button3.0),
            button4: big_endian::U16::new(keymap.button4.0),
            fx1: big_endian::U16::new(keymap.fx1.0),
            fx2: big_endian::U16::new(keymap.fx2.0),
            _unused2: [big_endian::U16::ZERO; 2],
        }
    }

    pub fn apply_keymap(&self, map: &mut Keymap) {
        map.start = Keycode::from(self.start.get());

        map.button1 = Keycode::from(self.button1.get());
        map.button2 = Keycode::from(self.button2.get());
        map.button3 = Keycode::from(self.button3.get());
        map.button4 = Keycode::from(self.button4.get());

        map.fx1 = Keycode::from(self.fx1.get());
        map.fx2 = Keycode::from(self.fx2.get());
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
