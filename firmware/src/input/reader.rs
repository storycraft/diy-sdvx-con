use crate::input::{KnobTurn, reader::button::ButtonInputRead};

pub mod button;
pub mod knob;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct InputRead {
    pub knobs: (KnobTurn, KnobTurn),
    pub buttons: ButtonInputRead,
}

impl InputRead {
    pub const DEFAULT: Self = Self {
        knobs: (KnobTurn::DEFAULT, KnobTurn::DEFAULT),
        buttons: ButtonInputRead::DEFAULT,
    };
}
