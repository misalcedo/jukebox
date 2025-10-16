use std::time::Duration;
use tokio::sync::watch::Receiver;
use url::Url;
use crate::{local, spotify};
use crate::progress::SongTracker;

pub struct Player {
    stream: spotify::Player,
    file: local::Player,
    last: Option<String>,
    tracker: SongTracker,
}

impl Player {
    fn new(stream: spotify::Player, file: local::Player) -> Self {
        Self {
            stream,
            file,
            last: None,
            tracker: SongTracker::default()
        }
    }

    pub async fn play(&mut self, input: String) -> anyhow::Result<()> {
        if self.last.as_ref() == Some(&input) && self.tracker.has_next() {
            if self.skip(&input).await? {
                self.tracker.start();
                return Ok(());
            }
        }

       let songs = self.play_uri(input.clone()).await?;

        self.last = Some(input);
        self.tracker.reset(songs);
        Ok(())
    }

    async fn skip(&mut self, input: &str) -> anyhow::Result<bool> {
        let uri = Url::parse(input)?;
        match uri.scheme() {
            "https" if uri.host_str() == Some("open.spotify.com") =>self.stream.skip().await,
            "spotify" => self.stream.skip().await,
            "file" => self.file.skip().await,
            _ => anyhow::bail!("Unknown scheme: {}", uri.scheme()),
        }
    }

    async fn play_uri(&mut self, input: String) -> anyhow::Result<Vec<Duration>> {
        let uri = Url::parse(&input)?;
        match uri.scheme() {
            "https" if uri.host_str() == Some("open.spotify.com") =>self.stream.play(input).await,
            "spotify" => self.stream.play(input).await,
            "file" => self.file.play(input).await,
            _ => anyhow::bail!("Unknown scheme: {}", uri.scheme()),
        }
    }

    pub async fn pause(&mut self) -> anyhow::Result<()> {
        match self.last.as_ref() {
            Some(last) => {
                let uri = Url::parse(last)?;
                match uri.scheme() {
                    "https" if uri.host_str() == Some("open.spotify.com") => self.stream.pause().await?,
                    "spotify" => self.stream.pause().await?,
                    "file" => self.file.pause().await?,
                    _ => anyhow::bail!("Unknown scheme: {}", uri.scheme()),
                }

                self.tracker.pause();
                Ok(())
            }
            None => {
                anyhow::bail!("Missing last url field");
            }
        }
    }
}

pub async fn run(
    mut receiver: Receiver<Option<String>>,
    stream: spotify::Player,
    file: local::Player,
) -> anyhow::Result<()> {
    let mut player = Player::new(stream, file);

    loop {
        receiver.changed().await?;

        let value = receiver.borrow_and_update().clone();

        tracing::debug!(?value, "received input");

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
