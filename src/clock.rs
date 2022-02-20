use std::time::{Duration, Instant};

const MHZ: f32 = 1_000_000.0;

// Oscillator generates a clock signal at a given frequency.
// It generally prevents the CPU from running "too fast" on modern hardware.
pub struct Oscillator {
    freq: Duration,
    start: Instant,
}

impl Oscillator {
    pub fn with_frequency(freq_mhz: f32) -> Oscillator {
        let freq = (freq_mhz * MHZ) as u64;
        Oscillator {
            freq: Duration::from_nanos(freq),
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
