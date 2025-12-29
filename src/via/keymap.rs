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
