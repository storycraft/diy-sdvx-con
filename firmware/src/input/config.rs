use embassy_rp::{Peri, peripherals::*};
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

pub const KNOB_SAMPLES: usize = 32;

pub type KnobFilter = filter::KnobFilter<32, 40>;
pub type ButtonDebouncer = filter::ButtonDebouncer<5>;
