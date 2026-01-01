pub struct KnobState {
    last: u16,
}

impl KnobState {
    /// Create new [`KnobState`] with initial knob value
    #[inline]
    pub const fn new(initial: u16) -> Self {
        Self { last: initial }
    }

    pub fn update(&mut self, now: u16) -> KnobTurn {
        let d = now as i16 - self.last as i16;

        let delta = if d > 2048 {
            d - 4096
        } else if d < -2048 {
            d + 4096
        } else {
            d
        };

        if delta.abs() <= 16 {
            return KnobTurn::None;
        }
        self.last = now;

        if delta < 0 {
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
