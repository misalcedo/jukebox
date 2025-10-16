use crate::spotify::models::{Album, Playlist, Track};
use std::fmt::{Display, Formatter};
use std::time::Duration;

pub enum Playable {
    Track(Track),
    Playlist(Playlist),
    Album(Album),
}

pub struct Song {
    pub uri: String,
    pub duration: Duration,
}

impl Playable {
    pub fn songs(&self) -> Vec<Song> {
        let mut songs = Vec::new();

        match self {
            Playable::Track(track) => {
                songs.push(Song {
                    uri: track.uri.clone(),
                    duration: Duration::from_millis(track.duration_ms),
                });
            }
            Playable::Playlist(playlist) => {
                songs.reserve(playlist.tracks.items.len());
                for item in playlist.tracks.items.iter() {
                    songs.push(Song {
                        uri: item.track.uri.clone(),
                        duration: Duration::from_millis(item.track.duration_ms),
                    });
                }
            }
            Playable::Album(album) => {
                if let Some(tracks) = &album.tracks {
                    songs.reserve(tracks.items.len());
                    for item in tracks.items.iter() {
                        songs.push(Song {
                            uri: item.uri.clone(),
                            duration: Duration::from_millis(item.duration_ms),
                        });
                    }
                }
            }
        };

        songs
    }
}

impl Display for Playable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Playable::Track(track) => write!(f, "Track: {}", track.name),
            Playable::Playlist(playlist) => write!(f, "Playlist: {}", playlist.name),
            Playable::Album(album) => write!(f, "Album: {}", album.name),
        }
    }
}
