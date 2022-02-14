use std::time::{Duration, Instant};

const MHZ: u64 = 1_000_000;

// Oscillator generates a clock signal at a given frequency.
// It generally prevents the CPU from running "too fast" on modern hardware.
pub struct Oscillator {
    freq: Duration,
    start: Instant,
}

impl Oscillator {
    pub fn with_mhz(freq: u64) -> Oscillator {
        Oscillator {
            freq: Duration::from_nanos(freq * MHZ),
            start: Instant::now(),
        }
    }
}

impl Iterator for Oscillator {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        while self.start.elapsed() < self.freq {}
        Some(())
    }
}
