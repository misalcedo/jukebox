mod client;
pub mod models;
mod uri;
pub mod playable;

use std::time::Duration;
use anyhow::anyhow;
use rand::prelude::SliceRandom;
use reqwest::StatusCode;

use crate::spotify::models::StartPlaybackRequest;
pub use playable::Playable;
pub use crate::spotify::client::Client;
use crate::spotify::uri::Uri;

pub struct Player {
    client: Client,
    preferred_device: Option<String>,
    device_id: Option<String>,
}

impl Player {
    pub fn new(client: Client, preferred_device: Option<String>) -> Self {
        Self {
            client,
            preferred_device,
            device_id: None,
        }
    }

    pub async fn play(&mut self, uri: String) -> anyhow::Result<Vec<Duration>> {
        if self.preferred_device.is_some() && self.device_id.is_none() {
            let preferred_device_name = self.preferred_device.clone().unwrap_or_default();
            self.device_id = Some(self.preferred_device_id(preferred_device_name).await?);
        }

        let playable = self.resolve_uri(&uri).await?;
        let mut songs = playable.songs();

        if songs.is_empty() {
            return Err(anyhow!("No songs to play"));
        }

        songs.shuffle(&mut rand::rng());

        let uris: Vec<String> = songs.iter().map(|song| song.uri.clone()).collect();
        let request = StartPlaybackRequest::from(uris);

        self.client.play(self.device_id.clone(), &request).await?;

        Ok(songs.iter().map(|song| song.duration).collect())
    }

    pub async fn skip(&mut self) -> anyhow::Result<bool> {
        match self.client.skip_to_next(None).await {
            Ok(_) => {
                Ok(true)
            }
            Err(e) if not_supported(e.status()) => {
                tracing::warn!(%e, "Failed to skip song, shuffling instead");
                Ok(false)
            }
            Err(e) => Err(anyhow::anyhow!(e)),
        }
    }

    pub async fn pause(&mut self) -> anyhow::Result<()> {
        if let Err(e) = self.client.pause(None).await {
            // Song may not be playing.
            if not_supported(e.status()) {
                return Ok(());
            }

            return Err(anyhow::anyhow!(e));
        };

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

    async fn preferred_device_id(&mut self, preferred_device: String) -> anyhow::Result<String> {
        match self
            .client
            .get_available_devices()
            .await?
            .devices
            .into_iter()
            .find(|device| device.name == preferred_device)
        {
            None => Err(anyhow!(
                "Found no matching device for {:?}",
                preferred_device
            )),
            Some(device) => Ok(device.id),
        }
    }
}

fn not_supported(status: Option<StatusCode>) -> bool {
    status == Some(StatusCode::NOT_FOUND) || status == Some(StatusCode::FORBIDDEN)
}
