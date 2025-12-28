pub struct KnobState {
    last: u8,
}

impl KnobState {
    /// Create new [`KnobState`] with initial knob value
    #[inline]
    pub const fn new(initial: u8) -> Self {
        Self { last: initial }
    }

    pub fn update(&mut self, now: u8) -> KnobTurn {
        let d = now as i16 - self.last as i16;
        self.last = now;

        let delta = if d > 127 {
            d - 256
        } else if d < -127 {
            d + 256
        } else {
            d
        } as i8;

        if delta == 0 {
            KnobTurn::None
        } else if delta < 0 {
            KnobTurn::Left
        } else {
            KnobTurn::Right
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum KnobTurn {
    #[default]
    None,
    Left,
    Right,
}
