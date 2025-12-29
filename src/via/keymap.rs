use num_traits::FromPrimitive;

use crate::{keycode::Keycode, userdata::keymap::Keymap};

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

/// Emulate qmk keymap.
/// The keymap buffer is inversed for easier endianess handling
pub fn keymap_buffer(map: &Keymap) -> [u16; 12] {
    [
        0,
        0,
        map.fx2 as u16,
        map.fx1 as u16,
        map.button4 as u16,
        map.button3 as u16,
        map.button2 as u16,
        map.button1 as u16,
        0,
        0,
        map.start as u16,
        0,
    ]
}

pub fn apply_keymap_buffer(map: &mut Keymap, buf: &[u16; 12]) {
    map.fx2 = Keycode::from_u16(buf[2]).unwrap_or_default();
    map.fx1 = Keycode::from_u16(buf[3]).unwrap_or_default();

    map.button4 = Keycode::from_u16(buf[4]).unwrap_or_default();
    map.button3 = Keycode::from_u16(buf[5]).unwrap_or_default();
    map.button2 = Keycode::from_u16(buf[6]).unwrap_or_default();
    map.button1 = Keycode::from_u16(buf[7]).unwrap_or_default();

    map.start = Keycode::from_u16(buf[10]).unwrap_or_default();
}
