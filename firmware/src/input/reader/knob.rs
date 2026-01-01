use embassy_time::Duration;

use crate::input::KnobTurn;

/// A filter to reduce noise in the knob value,
/// preventing the knob value from changing until it reaches the threshold or the throttle time has elapsed.
pub struct KnobFilter {
    last_value: u16,
    last_turn: KnobTurn,
    timer: Duration,
}

impl KnobFilter {
    /// Create new [`KnobState`] with initial knob value
    #[inline]
    pub const fn new(initial: u16) -> Self {
        Self {
            last_value: initial,
            last_turn: KnobTurn::None,
            timer: Duration::MIN,
        }
    }

    pub fn update(&mut self, raw_value: u16, elapsed: Duration) -> KnobTurn {
        const THRESHOLD_VALUE: i16 = 20;
        const THROTTLE_DURATION: Duration = Duration::from_millis(50);

        let delta = {
            let d = raw_value as i16 - self.last_value as i16;
            if d > 2048 {
                d - 4096
            } else if d < -2048 {
                d + 4096
            } else {
                d
            }
        };

        // Check if movement is in threshold range.
        if delta.abs() < THRESHOLD_VALUE {
            // Prevent changes in threshold range.
            if self.timer != Duration::MIN {
                self.timer = self.timer.checked_sub(elapsed).unwrap_or(Duration::MIN);
                return self.last_turn;
            }

            self.last_turn = KnobTurn::None;
            return KnobTurn::None;
        }

        self.last_value = raw_value;
        self.timer = THROTTLE_DURATION;
        if delta < 0 {
            self.last_turn = KnobTurn::Left;
            KnobTurn::Left
        } else {
            self.last_turn = KnobTurn::Right;
            KnobTurn::Right
        }
    }
}
