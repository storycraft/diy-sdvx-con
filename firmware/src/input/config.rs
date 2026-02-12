use embassy_rp::{
    Peri,
    adc::{self, Channel},
    gpio::{Input, Pin, Pull},
    peripherals::*,
};

use crate::input::reader::button::{Button, Buttons};
pub struct InputPinout<'a> {
    pub button1: Peri<'a, PIN_0>,
    pub button2: Peri<'a, PIN_1>,
    pub button3: Peri<'a, PIN_2>,
    pub button4: Peri<'a, PIN_3>,

    pub fx1: Peri<'a, PIN_4>,
    pub fx2: Peri<'a, PIN_5>,

    pub start: Peri<'a, PIN_6>,

    pub left_knob: Peri<'a, PIN_26>,
    pub right_knob: Peri<'a, PIN_27>,
}

impl<'a> InputPinout<'a> {
    pub fn inputs(self) -> (Buttons<'a>, [Channel<'a>; 2]) {
        (
            Buttons {
                button1: button(self.button1),
                button2: button(self.button2),
                button3: button(self.button3),
                button4: button(self.button4),
                fx1: button(self.fx1),
                fx2: button(self.fx2),
                start: button(self.start),
            },
            [
                adc::Channel::new_pin(self.left_knob, Pull::None),
                adc::Channel::new_pin(self.right_knob, Pull::None),
            ],
        )
    }
}

#[inline(always)]
fn button<'a>(pin: Peri<'a, impl Pin>) -> Button<'a> {
    let mut input = Input::new(pin, Pull::Up);
    input.set_schmitt(true);
    input.set_inversion(true);
    Button::new(input)
}

pub const KNOB_SAMPLES: usize = 32;
pub const MOUSE_CURSOR_SPEED: i8 = 3;
pub const MOUSE_WHEEL_SPEED: i8 = 1;

pub type KnobFilter = filter::KnobFilter<32, 10>;
pub type ButtonDebouncer = filter::ButtonDebouncer<5>;
