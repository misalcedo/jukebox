use crate::spotify::models::{Album, Playlist, Track};
use std::fmt::{Display, Formatter};

pub enum Playable {
    Track(Track),
    Playlist(Playlist),
    Album(Album),
}

impl Playable {
    pub fn name(&self) -> &str {
        match self {
            Playable::Track(track) => &track.name,
            Playable::Playlist(playlist) => &playlist.name,
            Playable::Album(album) => &album.name,
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            Playable::Track(_) => "Track",
            Playable::Playlist(_) => "Playlist",
            Playable::Album(_) => "Album",
        }
    }

    pub fn uris(&self) -> Vec<String> {
        let mut uris = Vec::new();

        match self {
            Playable::Track(track) => {
                uris.push(track.uri.clone());
            }
            Playable::Playlist(playlist) => {
                uris.reserve(playlist.tracks.items.len());
                for item in playlist.tracks.items.iter() {
                    uris.push(item.track.uri.clone());
                }
            }
            Playable::Album(album) => {
                if let Some(tracks) = &album.tracks {
                    uris.reserve(tracks.items.len());
                    for item in tracks.items.iter() {
                        uris.push(item.uri.clone());
                    }
                }
            }
        };

        uris
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