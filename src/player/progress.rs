use crate::player::playable::Song;
use std::time::Instant;

#[derive(Default)]
pub struct SongTracker {
    songs: Vec<Song>,
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

            while let Some(song) = self.songs.get(self.index) {
                match remaining.checked_sub(song.duration) {
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

    pub fn reset(&mut self, songs: Vec<Song>) {
        self.songs = songs;
        self.start = Some(Instant::now());
        self.index = 0;
    }

    pub fn has_next(&self) -> bool {
        self.index < self.songs.len()
    }
}
