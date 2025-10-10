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
        if self.last.as_deref() == Some(&input) && self.tracker.has_next() {
            // skip
            self.tracker.start();
        }

        let uri = Url::parse(&input)?;
        match uri.scheme() {
            "https" if uri.host_str() == Some("open.spotify.com") =>self.stream.play(input).await,
            "spotify" => self.stream.play(input).await,
            "file" => self.file.play(input).await,
            _ => anyhow::bail!("Unknown scheme: {}", uri.scheme()),
        }
    }

    pub async fn pause(&mut self) -> anyhow::Result<()> {
        match tokio::join!(self.file.pause(), self.stream.pause()) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(e1), Err(e2)) => anyhow::bail!("Failed to pause playback: {} {}", e1, e2),
            (Err(e), Ok(())) => Err(e),
            (Ok(()), Err(e)) => Err(e),
        }?;

        self.tracker.pause();

        Ok(())
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
