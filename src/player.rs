use crate::spotify::models::{Device, StartPlaybackRequest};
use crate::{card, cli, spotify, token};
use anyhow::anyhow;
use rand::prelude::SliceRandom;

mod playable;

pub use playable::Playable;

pub trait Observer {
    fn on_playback_started(&self, playable: Playable);

    fn on_playback_paused(&self);
}

impl Observer for () {
    fn on_playback_started(&self, playable: Playable) {
        tracing::info!(kind = playable.kind(), name = playable.name(), "Playing");
    }

    fn on_playback_paused(&self) {
        tracing::info!("Paused playback")
    }
}

pub struct Player<O> {
    observer: O,
}

impl<O> From<O> for Player<O>
where
    O: Observer,
{
    fn from(observer: O) -> Self {
        Self {
            observer,
        }
    }
}

impl Default for Player<()> {
    fn default() -> Self {
        Self::from(())
    }
}

impl<O> Player<O>
where
    O: Observer,
{
    pub fn run(self, arguments: &cli::Arguments) -> anyhow::Result<()> {
        let oauth = token::Client::new(arguments.client_id.clone(), arguments.token_cache.clone());
        let mut client = spotify::Client::new(oauth, arguments.market.clone());

        let device = choose_device(&mut client, arguments.device.as_deref())?;

        let mut device_id = Some(device.id);
        let mut request = StartPlaybackRequest::default();

        let ctx = pcsc::Context::establish(pcsc::Scope::User)?;
        let mut reader = choose_reader(ctx)?;

        loop {
            reader.wait(None)?;

            match reader.read() {
                Ok(None) => if let Err(e) = self.pause_playback(&mut client) {
                    tracing::error!(%e, "Failed to pause playback");
                },
                Ok(Some(uri)) if uri.is_empty() => {
                    tracing::info!("Read empty tag");
                }
                Ok(Some(uri)) => match start_playback(&mut client, &mut device_id, &mut request, &uri) {
                    Err(e) => {
                        tracing::error!(%e, %uri, "Failed to start playback");
                    }
                    Ok(playable) => {
                        self.observer.on_playback_started(playable);
                    }
                },
                Err(e) => {
                    tracing::warn!(%e, "Failed to read the URI from the card");
                }
            }
        }
    }

    fn pause_playback(&self, client: &mut spotify::Client) -> anyhow::Result<()> {
        // Song may not be playing.
        if let Err(e) = client.pause(None) {
            if e.status() == Some(reqwest::StatusCode::FORBIDDEN) {
                return Ok(());
            }
        };

        self.observer.on_playback_paused();

        Ok(())
    }
}

fn choose_reader(ctx: pcsc::Context) -> anyhow::Result<card::Reader> {
    for reader in ctx.list_readers_owned()? {
        if let Ok(name) = reader.to_str() {
            if name.contains("PICC") {
                return Ok(card::Reader::new(ctx, reader));
            }
        }
    }

    Err(anyhow!("No PICC readers are connected"))
}

fn choose_device(client: &mut spotify::Client, name: Option<&str>) -> anyhow::Result<Device> {
    match client
        .get_available_devices()?
        .devices
        .into_iter()
        .find(|device| match name {
            Some(name) => device.name == name,
            None => true,
        }) {
        None => {
            Err(anyhow!("Found no matching device for {:?}", name))
        }
        Some(device) => Ok(device)
    }
}

fn start_playback(
    client: &mut spotify::Client,
    device_id: &mut Option<String>,
    request: &mut StartPlaybackRequest,
    uri: &str,
) -> anyhow::Result<Playable> {
    let playable = resolve_uri(client, uri)?;
    let mut uris = playable.uris();

    if uris.is_empty() {
        return Err(anyhow!("No tracks to play"));
    }

    let mut current = None;
    if let Some(state) = client.get_playback_state()? {
        // Continue playback on the current device.
        device_id.replace(state.device.id);
        current = state.item.map(|item| item.uri);
    }

    // Skip to the next song if the current song is not the last song in the queue, but is part of the queue.
    let mut iterator = request.uris.iter().rev();
    if iterator.next() != current.as_ref() && iterator.any(|i| Some(i) == current.as_ref()) {
        client.skip_to_next(None)?;
        tracing::info!(%uri, name = playable.name(), %?current, %request.uris, "Skipping to the next song in the queue");
        return Ok(playable);
    }

    // Shuffle until the current song is not the first in the queue.
    let mut rng = rand::thread_rng();
    loop {
        uris.shuffle(&mut rng);
        if uris.first() != current.as_ref() {
            break;
        }
    }

    *request = StartPlaybackRequest::from(uris);
    client.play(device_id.clone(), request)?;

    tracing::info!(%uri, name = playable.name(), %?current, %request.uris, "Playing the songs resolved from the tag");

    Ok(playable)
}

fn resolve_uri(client: &mut spotify::Client, uri: &str) -> anyhow::Result<Playable> {
    let uri: spotify::Uri = uri.parse()?;

    match uri.category.as_str() {
        "track" => {
            Ok(Playable::Track(client.get_track(&uri.id)?))
        }
        "playlist" => {
            Ok(Playable::Playlist(client.get_playlist(&uri.id)?))
        }
        "album" => {
            Ok(Playable::Album(client.get_album(&uri.id)?))
        }
        _ => {
            Err(anyhow!("Unsupported URI category"))
        }
    }
}
