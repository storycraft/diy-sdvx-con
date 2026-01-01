use zerocopy::{Immutable, IntoBytes, TryFromBytes};

use crate::keycode::Keycode;

#[derive(Clone, PartialEq, Eq, TryFromBytes, IntoBytes, Immutable)]
#[repr(C)]
pub struct Keymap {
    pub left_knob_left: Keycode,
    pub left_knob_right: Keycode,

    pub start: Keycode,

    pub right_knob_left: Keycode,
    pub right_knob_right: Keycode,

    pub button1: Keycode,
    pub button2: Keycode,
    pub button3: Keycode,
    pub button4: Keycode,

    pub fx1: Keycode,
    pub fx2: Keycode,

    pub _unused: u16,
}

impl Keymap {
    pub const DEFAULT: Self = Self {
        left_knob_left: Keycode::DPAD_LEFT,
        left_knob_right: Keycode::DPAD_RIGHT,
        start: Keycode::JOY_BTN10,
        right_knob_left: Keycode::JOY_BTN1,
        right_knob_right: Keycode::JOY_BTN3,
        button1: Keycode::JOY_BTN7,
        button2: Keycode::JOY_BTN5,
        button3: Keycode::JOY_BTN6,
        button4: Keycode::JOY_BTN8,
        fx1: Keycode::DPAD_DOWN,
        fx2: Keycode::JOY_BTN2,
        _unused: 0,
    };
}

impl Default for Keymap {
    fn default() -> Self {
        Self::DEFAULT
    }
}
