use crate::input::reader::button::ButtonInputRead;

pub mod button;
pub mod knob;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct InputRead {
    pub knobs: (i16, i16),
    pub buttons: ButtonInputRead,
}

impl InputRead {
    pub const DEFAULT: Self = Self {
        knobs: (0, 0),
        buttons: ButtonInputRead::DEFAULT,
    };
}
