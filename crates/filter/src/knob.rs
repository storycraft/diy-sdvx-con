/// A filter to reduce noise in the knob value,
/// preventing the knob value from changing until it reaches the threshold or the throttle time has elapsed.
pub struct KnobFilter<const THRESHOLD_VALUE: i16, const THROTTLE_DURATION_MS: u16> {
    last_raw_value: u16,
    filtered_delta: i16,
    timer: u16,
}

impl<const THRESHOLD_VALUE: i16, const THROTTLE_DURATION_MS: u16>
    KnobFilter<THRESHOLD_VALUE, THROTTLE_DURATION_MS>
{
    /// Create new [`KnobFilter`] with initial knob value
    #[inline]
    pub const fn new(initial: u16) -> Self {
        Self {
            last_raw_value: initial,
            filtered_delta: 0,
            timer: 0,
        }
    }

    pub fn filter(&mut self, raw_value: u16, elapsed_ms: u16) -> i16 {
        let delta = {
            let d = raw_value as i16 - self.last_raw_value as i16;
            if d >= 2048 {
                d - 4096
            } else if d <= -2048 {
                d + 4096
            } else {
                d
            }
        };

        // Check if movement is in threshold range.
        if delta.abs() < THRESHOLD_VALUE {
            // Prevent changes in throttle time.
            if self.timer != 0 {
                self.timer = self.timer.checked_sub(elapsed_ms).unwrap_or(0);
                return self.filtered_delta;
            }

            self.filtered_delta = delta;
            return delta;
        }

        self.last_raw_value = raw_value;
        self.timer = THROTTLE_DURATION_MS;
        self.filtered_delta = delta;
        delta
    }
}
