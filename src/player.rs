use crate::{spotify, token};
use anyhow::anyhow;
use rand::prelude::SliceRandom;

mod playable;

use crate::cli::Arguments;
use crate::spotify::models::StartPlaybackRequest;
use crate::spotify::Uri;
pub use playable::Playable;

pub struct Player {
    client: spotify::Client,
    preferred_device: Option<String>,
    device_id: Option<String>,
    uris: Vec<String>,
}

impl TryFrom<Arguments> for Player {
    type Error = anyhow::Error;

    fn try_from(arguments: Arguments) -> Result<Self, Self::Error> {
        let oauth = token::Client::new(arguments.client_id.clone(), arguments.token_cache.clone());
        let client = spotify::Client::new(oauth, arguments.market.clone());

        Ok(Self {
            client,
            preferred_device: arguments.device,
            device_id: None,
            uris: vec![],
        })
    }
}

impl Player {
    pub async fn play(&mut self, uri: String) -> anyhow::Result<()> {
        let playable = self.resolve_uri(&uri).await?;
        let mut tracks = playable.uris();

        if tracks.is_empty() {
            return Err(anyhow!("No tracks to play"));
        }

        if self.device_id.is_none() {
            let device_id = self.preferred_device().await?;
            self.device_id = Some(device_id);
        }

        let current = self
            .client
            .get_playback_state()
            .await?
            .and_then(|state| state.item.map(|item| item.uri));

        match current {
            Some(track) if tracks.contains(&track) => {
                self.uris.rotate_left(1);
                while self.uris.first() == Some(&track) {
                    self.uris.rotate_left(1);
                }
            }
            _ => {
                tracks.shuffle(&mut rand::thread_rng());
                self.uris = tracks;
            }
        }

        let request = StartPlaybackRequest::from(self.uris.clone());
        self.client.play(self.device_id.clone(), &request).await?;

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

        Ok(())
    }

    async fn resolve_uri(&mut self, uri: &str) -> anyhow::Result<Playable> {
        let uri: spotify::Uri = uri.parse()?;

        match uri.category.as_str() {
            "track" => Ok(Playable::Track(self.client.get_track(&uri.id).await?)),
            "playlist" if uri.mystery => {
                let playable = Playable::Playlist(self.client.get_playlist(&uri.id).await?);
                let mut tracks = playable.uris();

                tracks.shuffle(&mut rand::thread_rng());

                let track = tracks.first().ok_or_else(|| anyhow!("No tracks in playlist"))?;
                let track_uri: Uri = track.parse()?;

                Ok(Playable::Track(self.client.get_track(&track_uri.id).await?))
            }
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
