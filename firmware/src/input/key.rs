pub mod builder;

use keycode::Keycode;
use usbd_hid::descriptor::{KeyboardReport, MediaKeyboardReport};

use crate::{
    input::{
        key::builder::{GamepadInputBuilder, MouseInputBuilder},
        report,
    },
    keycodes,
};

#[derive(Default)]
pub struct InputReports {
    gamepad: Option<GamepadInputBuilder>,
    keyboard: Option<KeyboardReport>,
    media: Option<MediaKeyboardReport>,
    mouse: Option<MouseInputBuilder>,
}

impl InputReports {
    pub fn send(self) {
        if let Some(gamepad) = self.gamepad {
            report::GAMEPAD.signal(gamepad.build());
        }

        if let Some(keyboard) = self.keyboard {
            report::KEYBOARD.signal(keyboard);
        }

        if let Some(mouse) = self.mouse {
            report::MOUSE.signal(mouse.build());
        }

        if let Some(media) = self.media {
            report::MEDIA.signal(media);
        }
    }

    pub fn press_key(&mut self, code: Keycode) {
        const GAMEPAD_KEY_START: u16 = keycodes::JOY_BTN1.0;
        const GAMEPAD_KEY_END: u16 = keycodes::DPAD_RIGHT.0;

        const MOUSE_KEY_START: u16 = Keycode::QK_MOUSE_CURSOR_UP.0;
        const MOUSE_KEY_END: u16 = Keycode::QK_MOUSE_ACCELERATION_2.0;

        match code.0 {
            Keycode::RANGE_QK_BASIC_START..MOUSE_KEY_START => {}

            MOUSE_KEY_START..=MOUSE_KEY_END => self.mouse(code),

            GAMEPAD_KEY_START..=GAMEPAD_KEY_END => self.gamepad(code),

            _ => {}
        }
    }

    #[inline(always)]
    fn gamepad(&mut self, code: Keycode) {
        match code {
            keycodes::DPAD_UP => {
                self.gamepad.get_or_insert_default().dpad_up();
            }

            keycodes::DPAD_LEFT => {
                self.gamepad.get_or_insert_default().dpad_left();
            }

            keycodes::DPAD_DOWN => {
                self.gamepad.get_or_insert_default().dpad_down();
            }

            keycodes::DPAD_RIGHT => {
                self.gamepad.get_or_insert_default().dpad_right();
            }

            Keycode(offset) => {
                self.gamepad
                    .get_or_insert_default()
                    .button((offset - keycodes::JOY_BTN1.0) as u8);
            }
        }
    }

    #[inline(always)]
    fn mouse(&mut self, code: Keycode) {
        const BUTTON_START: u16 = Keycode::QK_MOUSE_BUTTON_1.0;
        const BUTTON_END: u16 = Keycode::QK_MOUSE_BUTTON_8.0;

        match code {
            Keycode::QK_MOUSE_CURSOR_UP => {
                self.mouse.get_or_insert_default().cursor_up();
            }

            Keycode::QK_MOUSE_CURSOR_DOWN => {
                self.mouse.get_or_insert_default().cursor_down();
            }

            Keycode::QK_MOUSE_CURSOR_LEFT => {
                self.mouse.get_or_insert_default().cursor_left();
            }

            Keycode::QK_MOUSE_CURSOR_RIGHT => {
                self.mouse.get_or_insert_default().cursor_right();
            }

            Keycode::QK_MOUSE_WHEEL_UP => {
                self.mouse.get_or_insert_default().wheel_up();
            }

            Keycode::QK_MOUSE_WHEEL_DOWN => {
                self.mouse.get_or_insert_default().wheel_down();
            }

            Keycode::QK_MOUSE_WHEEL_LEFT => {
                self.mouse.get_or_insert_default().wheel_left();
            }

            Keycode::QK_MOUSE_WHEEL_RIGHT => {
                self.mouse.get_or_insert_default().wheel_right();
            }

            Keycode(BUTTON_START..=BUTTON_END) => {
                self.mouse
                    .get_or_insert_default()
                    .button((code.0 - BUTTON_START) as _);
            }

            _ => {}
        }
    }
}
