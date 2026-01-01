use embassy_time::Duration;

/// Debouncer for button.
///
/// The debouncer algorithm instantly react to button press,
/// but apply defer debouncing on release.
/// It gives lowest latency and some noise resistance.
/// TODO:: remove [`Duration`]
pub struct ButtonDebouncer<const DEBOUNCE_MS: u8> {
    release_last_raw_value: bool,
    debounced: bool,
    timer: Duration,
}

impl<const DEBOUNCE_MS: u8> ButtonDebouncer<DEBOUNCE_MS> {
    pub const fn new(initial: bool) -> Self {
        Self {
            release_last_raw_value: initial,
            debounced: initial,
            timer: Duration::MAX,
        }
    }

    pub fn debounce(&mut self, raw_state: bool, elapsed: Duration) -> bool {
        if self.timer != Duration::MIN {
            self.timer = self.timer.checked_sub(elapsed).unwrap_or(Duration::MIN);
        }

        // Reset timer if raw state changes during release debounce time
        if self.debounced && self.release_last_raw_value != raw_state {
            self.timer = Duration::from_millis(DEBOUNCE_MS as _);
            self.release_last_raw_value = raw_state;
            return true;
        }

        match (raw_state, self.debounced) {
            (true, true) | (false, false) => raw_state,

            // From released to pressed.
            // Changes instantly
            (true, false) => {
                self.debounced = true;
                true
            }

            // From pressed to released.
            // Wait for timer and changes
            (false, true) => {
                // If timer is not finished, block changes.
                if self.timer > Duration::MIN {
                    return true;
                }

                self.debounced = false;
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use embassy_time::Duration;

    use crate::input::debouncer::ButtonDebouncer;

    #[test]
    fn debouncer_test() {
        // (raw_state, debounced_state)
        let input_seq = [
            // Press start (instantly change)
            (false, false),
            (true, true),
            (false, true),
            (false, true),
            // Pressed state
            (true, true),
            (true, true),
            (true, true),
            (true, true),
            // Release start (wait for 5ms to change)
            (false, true),
            (false, true),
            (true, true),
            (false, true),
            (false, true),
            (false, true),
            (false, true),
            (false, true),
            // Release end (debounce applied)
            (false, false),
            (false, false),
        ];

        let mut debouncer = ButtonDebouncer::<5>::new(false);
        for (i, (raw_state, debounced_state)) in input_seq.into_iter().enumerate() {
            println!("{i}");
            assert_eq!(
                debouncer.debounce(raw_state, Duration::from_millis(1)),
                debounced_state
            );
        }
    }
}
