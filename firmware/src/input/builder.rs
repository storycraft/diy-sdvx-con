use usbd_hid::descriptor::{KeyboardReport, MouseReport};

use crate::{input::config, usb::hid::GamepadInputReport};

#[derive(Default)]
pub struct GamepadInputBuilder(GamepadInputReport);

impl GamepadInputBuilder {
    #[inline]
    pub fn button(&mut self, n: u8) {
        self.0.buttons |= 1 << n;
    }

    #[inline]
    pub fn dpad_up(&mut self) {
        self.0.dpad = match self.0.dpad {
            // centered
            0 => 1,
            // up
            1 | 2 | 8 => self.0.dpad,
            // left
            7 => 8,
            // right
            3 => 2,
            _ => 0,
        };
    }

    #[inline]
    pub fn dpad_down(&mut self) {
        self.0.dpad = match self.0.dpad {
            // centered
            0 => 5,
            // down
            4..=6 => self.0.dpad,
            // left
            7 => 6,
            // right
            3 => 4,
            _ => 0,
        };
    }

    #[inline]
    pub fn dpad_left(&mut self) {
        self.0.dpad = match self.0.dpad {
            // centered
            0 => 7,
            // left
            6..=8 => self.0.dpad,
            // up
            1 => 8,
            // down
            5 => 6,
            _ => 0,
        };
    }

    #[inline]
    pub fn dpad_right(&mut self) {
        self.0.dpad = match self.0.dpad {
            // centered
            0 => 3,
            // left
            2..=4 => self.0.dpad,
            // up
            1 => 2,
            // down
            5 => 4,
            _ => 0,
        };
    }

    #[inline]
    pub const fn build(self) -> GamepadInputReport {
        self.0
    }
}

pub struct MouseInputBuilder(MouseReport);

impl MouseInputBuilder {
    #[inline]
    pub fn cursor_up(&mut self) {
        self.0.y -= config::MOUSE_CURSOR_SPEED;
    }

    #[inline]
    pub fn cursor_down(&mut self) {
        self.0.y += config::MOUSE_CURSOR_SPEED;
    }

    #[inline]
    pub fn cursor_left(&mut self) {
        self.0.x -= config::MOUSE_CURSOR_SPEED;
    }

    #[inline]
    pub fn cursor_right(&mut self) {
        self.0.x += config::MOUSE_CURSOR_SPEED;
    }

    #[inline]
    pub fn wheel_up(&mut self) {
        self.0.wheel += config::MOUSE_WHEEL_SPEED;
    }

    #[inline]
    pub fn wheel_down(&mut self) {
        self.0.wheel -= config::MOUSE_WHEEL_SPEED;
    }

    #[inline]
    pub fn wheel_left(&mut self) {
        self.0.pan += config::MOUSE_WHEEL_SPEED;
    }

    #[inline]
    pub fn wheel_right(&mut self) {
        self.0.pan -= config::MOUSE_WHEEL_SPEED;
    }

    #[inline]
    pub fn button(&mut self, n: u8) {
        self.0.buttons |= 1 << n;
    }

    #[inline]
    pub const fn build(self) -> MouseReport {
        self.0
    }
}

impl Default for MouseInputBuilder {
    fn default() -> Self {
        Self(MouseReport {
            buttons: 0,
            x: 0,
            y: 0,
            wheel: 0,
            pan: 0,
        })
    }
}

pub struct KeyboardInputBuilder {
    inner: KeyboardReport,
    next_key_index: usize,
}

impl KeyboardInputBuilder {
    #[inline]
    pub fn key(&mut self, code: u8) {
        let index = self.next_key_index;
        if index >= self.inner.keycodes.len() {
            return;
        }
        self.next_key_index += 1;
        self.inner.keycodes[index] = code;
    }

    #[inline]
    pub fn modifier(&mut self, n: u8) {
        self.inner.modifier |= 1 << n;
    }

    #[inline]
    pub const fn build(self) -> KeyboardReport {
        self.inner
    }
}

impl Default for KeyboardInputBuilder {
    fn default() -> Self {
        Self {
            inner: KeyboardReport {
                modifier: 0,
                reserved: 0,
                leds: 0,
                keycodes: [0; _],
            },
            next_key_index: 0,
        }
    }
}
