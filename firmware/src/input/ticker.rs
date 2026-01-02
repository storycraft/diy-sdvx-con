use embassy_time::Instant;

pub struct ElapsedTimer {
    last: Instant,
}

impl ElapsedTimer {
    pub const fn new(initial: Instant) -> Self {
        Self { last: initial }
    }

    pub fn next_elapsed_ms(&mut self) -> u16 {
        let now = Instant::now();
        let elapsed_ms = now.duration_since(self.last).as_millis().min(u16::MAX as _) as u16;
        if elapsed_ms > 0 {
            // Round down last time
            self.last = Instant::from_millis(now.as_millis());
        }

        elapsed_ms
    }
}
