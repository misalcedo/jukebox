use crate::{spotify, token};
use anyhow::anyhow;
use rand::prelude::SliceRandom;
use reqwest::StatusCode;
use tokio::sync::watch::Receiver;

mod playable;
mod progress;

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

impl Player {
    fn new(oauth: token::Client, market: String, preferred_device: Option<String>) -> Self {
        let client = spotify::Client::new(oauth, market);

        Self {
            client,
            preferred_device,
            device_id: String::default(),
            last: None,
            tracker: SongTracker::default(),
        }
    }

    pub async fn play(&mut self, uri: String) -> anyhow::Result<()> {
        if self.device_id.is_empty() {
            self.device_id = self.preferred_device().await?;
        }

        let playable = self.resolve_uri(&uri).await?;
        let mut songs = playable.songs();

        if songs.is_empty() {
            return Err(anyhow!("No songs to play"));
        }

        if self.last.as_deref() == Some(playable.uri()) && self.tracker.has_next() {
            match self.client.skip_to_next(None).await {
                Ok(_) => {
                    self.tracker.start();
                    return Ok(());
                }
                Err(e) if not_supported(e.status()) => {
                    tracing::warn!(%e, "Failed to skip song, shuffling instead");
                    // fall through to still play the song instead of skipping
                }
                Err(e) => return Err(anyhow::anyhow!(e)),
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
            if not_supported(e.status()) {
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

pub async fn run(
    mut receiver: Receiver<Option<String>>,
    oauth: token::Client,
    market: String,
    preferred_device: Option<String>,
) -> anyhow::Result<()> {
    let mut player = Player::new(oauth, market, preferred_device);

    loop {
        receiver.changed().await?;

        let value = receiver.borrow_and_update().clone();

        match value {
            Some(uri) => {
                if let Err(e) = player.play(uri).await {
                    tracing::error!(%e, "Failed to start playback");
                }
            }
            None => {
                if let Err(e) = player.pause().await {
                    tracing::error!(%e, "Failed to pause playback");
                }
            }
        };
    }
}

fn not_supported(status: Option<StatusCode>) -> bool {
    status == Some(StatusCode::NOT_FOUND) || status == Some(StatusCode::FORBIDDEN)
}
