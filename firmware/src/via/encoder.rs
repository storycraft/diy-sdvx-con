use crate::{keycode::Keycode, userdata};

pub fn get_encoder_keycode(id: u8, clockwise: bool) -> Option<Keycode> {
    userdata::get(|data| match (id, clockwise) {
        (1, false) => Some(data.keymap.left_knob_left),
        (1, true) => Some(data.keymap.left_knob_right),

        (2, false) => Some(data.keymap.right_knob_left),
        (2, true) => Some(data.keymap.right_knob_right),

        _ => None,
    })
}

pub fn set_encoder_keycode(id: u8, clockwise: bool, code: Keycode) {
    userdata::update(|data| match (id, clockwise) {
        (1, false) => {
            data.keymap.left_knob_left = code;
        }
        (1, true) => {
            data.keymap.left_knob_right = code;
        }

        (2, false) => {
            data.keymap.right_knob_left = code;
        }
        (2, true) => {
            data.keymap.right_knob_right = code;
        }

        _ => {}
    })
}
