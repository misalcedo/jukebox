use std::time::{Duration, Instant};

#[derive(Default)]
pub struct StopWatch {
    start: Option<Instant>,
    duration: Duration,
    laps: usize,
}

impl StopWatch {
    pub fn start(&mut self) {
        if self.start.is_none() {
            self.start = Some(Instant::now());
        }
    }

    pub fn pause(&mut self) {
        if let Some(instant) = self.start.take() {
            self.duration += instant.elapsed();
            self.laps += 1;
        }
    }

    pub fn reset(&mut self) {
        self.start = Some(Instant::now());
        self.duration = Duration::default();
        self.laps = 0;
    }

    pub fn elapsed(&self) -> Duration {
        self.duration
    }

    pub fn laps(&self) -> usize {
        self.laps
    }
}