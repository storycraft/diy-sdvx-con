use keycode::Keycode;
use zerocopy::{Immutable, IntoBytes, TryFromBytes};

use crate::keycodes;

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
        left_knob_left: keycodes::DPAD_LEFT,
        left_knob_right: keycodes::DPAD_RIGHT,
        start: keycodes::JOY_BTN10,
        right_knob_left: keycodes::JOY_BTN1,
        right_knob_right: keycodes::JOY_BTN3,
        button1: keycodes::JOY_BTN7,
        button2: keycodes::JOY_BTN5,
        button3: keycodes::JOY_BTN6,
        button4: keycodes::JOY_BTN8,
        fx1: keycodes::DPAD_DOWN,
        fx2: keycodes::JOY_BTN2,
        _unused: 0,
    };
}

impl Default for Keymap {
    fn default() -> Self {
        Self::DEFAULT
    }
}
