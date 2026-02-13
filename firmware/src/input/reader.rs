use filter::KnobValue;

use crate::input::reader::button::ButtonInputRead;

pub mod button;
pub mod knob;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct InputRead {
    pub knobs: (KnobValue, KnobValue),
    pub buttons: ButtonInputRead,
}

impl InputRead {
    pub const DEFAULT: Self = Self {
        knobs: (KnobValue::DEFAULT, KnobValue::DEFAULT),
        buttons: ButtonInputRead::DEFAULT,
    };
}
