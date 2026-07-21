use std::time::{Duration, Instant};

#[derive(Default)]
pub struct SongTracker {
    songs: Vec<Duration>,
    start: Option<Instant>,
    index: usize,
}

impl SongTracker {
    pub fn start(&mut self) {
        if self.start.is_none() {
            self.start = Some(Instant::now());
        }
    }

    pub fn pause(&mut self) {
        if let Some(instant) = self.start.take() {
            let mut remaining = instant.elapsed();

            while let Some(duration) = self.songs.get(self.index) {
                match remaining.checked_sub(*duration) {
                    Some(difference) => {
                        self.index += 1;
                        remaining = difference;
                    }
                    None => break,
                }
            }

            self.index += 1;
        }
    }

    pub fn reset(&mut self, songs: Vec<Duration>) {
        self.songs = songs;
        self.start = Some(Instant::now());
        self.index = 0;
    }

    pub fn has_next(&self) -> bool {
        self.index < self.songs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::SongTracker;
    use std::time::{Duration, Instant};

    #[test]
    fn reset_starts_tracker_with_first_song() {
        let mut tracker = SongTracker::default();
        tracker.reset(vec![Duration::from_secs(1)]);

        assert!(tracker.start.is_some());
        assert_eq!(tracker.index, 0);
        assert!(tracker.has_next());
    }

    #[test]
    fn pause_does_not_change_index_when_not_started() {
        let mut tracker = SongTracker::default();
        tracker.songs = vec![Duration::from_secs(5)];
        tracker.pause();

        assert_eq!(tracker.index, 0);
        assert!(tracker.has_next());
    }

    #[test]
    fn pause_moves_to_next_song() {
        let mut tracker = SongTracker::default();
        tracker.songs = vec![Duration::from_secs(60)];
        tracker.start = Some(Instant::now());
        tracker.pause();

        assert_eq!(tracker.index, 1);
        assert!(!tracker.has_next());
    }
}
