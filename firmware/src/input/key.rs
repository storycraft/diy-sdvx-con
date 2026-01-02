use keycode::Keycode;
use usbd_hid::descriptor::MediaKeyboardReport;

use crate::{
    input::{
        builder::{GamepadInputBuilder, KeyboardInputBuilder, MouseInputBuilder},
        report,
    },
    keycodes,
};

#[derive(Default)]
pub struct InputReports {
    gamepad: Option<GamepadInputBuilder>,
    keyboard: Option<KeyboardInputBuilder>,
    media: Option<MediaKeyboardReport>,
    mouse: Option<MouseInputBuilder>,
}

impl InputReports {
    pub fn send(self) {
        if let Some(gamepad) = self.gamepad {
            report::GAMEPAD.signal(gamepad.build());
        }

        if let Some(keyboard) = self.keyboard {
            report::KEYBOARD.signal(keyboard.build());
        }

        if let Some(mouse) = self.mouse {
            report::MOUSE.signal(mouse.build());
        }

        if let Some(media) = self.media {
            report::MEDIA.signal(media);
        }
    }

    pub fn key(&mut self, code: Keycode, pressed: bool) {
        const GAMEPAD_KEY_START: u16 = keycodes::JOY_BTN1.0;
        const GAMEPAD_KEY_END: u16 = keycodes::DPAD_RIGHT.0;

        const MOUSE_KEY_START: u16 = Keycode::QK_MOUSE_CURSOR_UP.0;
        const MOUSE_KEY_END: u16 = Keycode::QK_MOUSE_ACCELERATION_2.0;

        match code.0 {
            Keycode::RANGE_QK_BASIC_START..MOUSE_KEY_START => {
                self.keyboard(code, pressed);
            }
            MOUSE_KEY_START..=MOUSE_KEY_END => self.mouse(code, pressed),

            GAMEPAD_KEY_START..=GAMEPAD_KEY_END => self.gamepad(code, pressed),

            _ => {}
        }
    }

    #[inline(always)]
    fn gamepad(&mut self, code: Keycode, pressed: bool) {
        const BUTTON_START: u16 = keycodes::JOY_BTN1.0;
        const BUTTON_END: u16 = keycodes::JOY_BTN16.0;

        let gamepad = self.gamepad.get_or_insert_default();
        if !pressed {
            return;
        }
        match code {
            keycodes::DPAD_UP => {
                gamepad.dpad_up();
            }

            keycodes::DPAD_LEFT => {
                gamepad.dpad_left();
            }

            keycodes::DPAD_DOWN => {
                gamepad.dpad_down();
            }

            keycodes::DPAD_RIGHT => {
                gamepad.dpad_right();
            }

            Keycode(BUTTON_START..=BUTTON_END) => {
                gamepad.button((code.0 - keycodes::JOY_BTN1.0) as u8);
            }

            _ => {}
        }
    }

    #[inline(always)]
    fn mouse(&mut self, code: Keycode, pressed: bool) {
        const BUTTON_START: u16 = Keycode::QK_MOUSE_BUTTON_1.0;
        const BUTTON_END: u16 = Keycode::QK_MOUSE_BUTTON_8.0;

        let mouse = self.mouse.get_or_insert_default();
        if !pressed {
            return;
        }
        match code {
            Keycode::QK_MOUSE_CURSOR_UP => {
                mouse.cursor_up();
            }

            Keycode::QK_MOUSE_CURSOR_DOWN => {
                mouse.cursor_down();
            }

            Keycode::QK_MOUSE_CURSOR_LEFT => {
                mouse.cursor_left();
            }

            Keycode::QK_MOUSE_CURSOR_RIGHT => {
                mouse.cursor_right();
            }

            Keycode::QK_MOUSE_WHEEL_UP => {
                mouse.wheel_up();
            }

            Keycode::QK_MOUSE_WHEEL_DOWN => {
                mouse.wheel_down();
            }

            Keycode::QK_MOUSE_WHEEL_LEFT => {
                mouse.wheel_left();
            }

            Keycode::QK_MOUSE_WHEEL_RIGHT => {
                mouse.wheel_right();
            }

            Keycode(BUTTON_START..=BUTTON_END) => {
                mouse.button((code.0 - BUTTON_START) as _);
            }

            _ => {}
        }
    }

    fn keyboard(&mut self, code: Keycode, pressed: bool) {
        const SCAN_CODE_START: u16 = Keycode::KC_A.0;
        const SCAN_CODE_END: u16 = Keycode::KC_EXSEL.0;
        const MODIFIER_START: u16 = Keycode::KC_LEFT_CTRL.0;
        const MODIFIER_END: u16 = Keycode::KC_RIGHT_GUI.0;
        const _: () = const { assert!(1 + MODIFIER_END - MODIFIER_START == 8) };

        let keyboard = self.keyboard.get_or_insert_default();
        if !pressed {
            return;
        }
        match code {
            Keycode(SCAN_CODE_START..=SCAN_CODE_END) => {
                keyboard.key(code.0 as u8);
            }

            Keycode(MODIFIER_START..=MODIFIER_END) => {
                keyboard.modifier((code.0 - MODIFIER_START) as u8);
            }

            _ => {}
        }
    }
}
