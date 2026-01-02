use crate::input::{KnobTurn, reader::button::ButtonInputRead};

pub mod button;
pub mod knob;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct InputRead {
    pub knobs: (KnobTurn, KnobTurn),
    pub buttons: ButtonInputRead,
}
