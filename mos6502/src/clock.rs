use std::time::{Duration, Instant};

const MHZ: f32 = 1_000_000.0;

// Oscillator generates a clock signal at a given frequency.
// It generally prevents the CPU from running "too fast" on modern hardware.
pub struct Oscillator {
    freq: Duration,
    start: Instant,
}

impl Oscillator {
    pub fn with_frequency(freq: f32) -> Oscillator {
        Oscillator {
            freq: Duration::from_nanos((MHZ / freq) as u64),
            start: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        while self.start.elapsed() < self.freq {}
        self.start = Instant::now();
    }
}

impl Iterator for Oscillator {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        self.tick();
        Some(())
    }
}
