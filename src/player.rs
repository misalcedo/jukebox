use crate::{spotify, token};
use anyhow::anyhow;
use rand::prelude::SliceRandom;

mod playable;
mod progress;

use crate::cli::Arguments;
use crate::player::progress::SongTracker;
use crate::spotify::models::StartPlaybackRequest;
use crate::spotify::Uri;
pub use playable::Playable;

pub struct Player {
    client: spotify::Client,
    preferred_device: Option<String>,
    device_id: String,
    last: Option<String>,
    tracker: SongTracker,
}

impl TryFrom<Arguments> for Player {
    type Error = anyhow::Error;

    fn try_from(arguments: Arguments) -> Result<Self, Self::Error> {
        let oauth = token::Client::new(arguments.client_id.clone(), arguments.token_cache.clone());
        let client = spotify::Client::new(oauth, arguments.market.clone());

        Ok(Self {
            client,
            preferred_device: arguments.device,
            device_id: String::default(),
            last: None,
            tracker: SongTracker::default(),
        })
    }
}

impl Player {
    pub async fn play(&mut self, uri: String) -> anyhow::Result<()> {
        if self.device_id.is_empty() {
            self.device_id = self.preferred_device().await?;
        }

        let playable = self.resolve_uri(&uri).await?;
        let mut songs = playable.songs();

        if songs.is_empty() {
            return Err(anyhow!("No songs to play"));
        }

        if self.last.as_deref() == Some(playable.uri()) {
            if self.tracker.has_next() {
                self.client.skip_to_next(None).await?;
                self.tracker.start();
                return Ok(());
            }
        }

        songs.shuffle(&mut rand::thread_rng());

        let uris: Vec<String> = songs.iter().map(|song| song.uri.clone()).collect();
        let request = StartPlaybackRequest::from(uris);

        self.client
            .play(Some(self.device_id.clone()), &request)
            .await?;
        self.last = Some(playable.uri().to_string());
        self.tracker.reset(songs);

        Ok(())
    }

    pub async fn pause(&mut self) -> anyhow::Result<()> {
        if let Err(e) = self.client.pause(None).await {
            // Song may not be playing.
            if e.status() == Some(reqwest::StatusCode::FORBIDDEN) {
                return Ok(());
            }

            return Err(anyhow::anyhow!(e));
        };

        self.tracker.pause();

        Ok(())
    }

    async fn resolve_uri(&mut self, uri: &str) -> anyhow::Result<Playable> {
        let uri: Uri = uri.parse()?;

        match uri.category.as_str() {
            "track" => Ok(Playable::Track(self.client.get_track(&uri.id).await?)),
            "playlist" => Ok(Playable::Playlist(self.client.get_playlist(&uri.id).await?)),
            "album" => Ok(Playable::Album(self.client.get_album(&uri.id).await?)),
            _ => Err(anyhow!("Unsupported URI category")),
        }
    }

    async fn preferred_device(&mut self) -> anyhow::Result<String> {
        match self
            .client
            .get_available_devices()
            .await?
            .devices
            .into_iter()
            .find(|device| match self.preferred_device.as_deref() {
                Some(name) => device.name == name,
                None => true,
            }) {
            None => Err(anyhow!(
                "Found no matching device for {:?}",
                self.preferred_device.as_deref()
            )),
            Some(device) => Ok(device.id),
        }
    }
}
