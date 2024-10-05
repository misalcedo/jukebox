use crate::spotify::models::{Album, Device, Playlist, StartPlaybackRequest, Track};
use crate::{card, cli, spotify, token};
use anyhow::anyhow;
use rand::prelude::SliceRandom;
use std::collections::HashSet;

pub trait Observer {
    fn on_playback_started(&self, playable: Playable);

    fn on_playback_paused(&self);
}

impl Observer for () {
    fn on_playback_started(&self, _: Playable) {}

    fn on_playback_paused(&self) {}
}

pub enum Playable {
    Track(Track),
    Playlist(Playlist),
    Album(Album),
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
            observer
        }
    }
}

impl Default for Player<()> {
    fn default() -> Self {
        Self {
            observer: (),
        }
    }
}

impl<O> Player<O>
where
    O: Observer,
{
    pub fn run(&self, arguments: &cli::Arguments) -> anyhow::Result<()> {
        let oauth = token::Client::new(arguments.client_id.clone(), arguments.token_cache.clone());
        let mut client = spotify::Client::new(oauth, arguments.market.clone());

        let device = choose_device(&mut client, arguments.device.as_deref())?;

        let ctx = pcsc::Context::establish(pcsc::Scope::User)?;
        let mut reader = choose_reader(ctx)?;

        loop {
            reader.wait(None)?;

            match reader.read() {
                Ok(None) => match pause_playback(&mut client, &self.observer) {
                    Ok(_) => tracing::info!("Paused playback"),
                    Err(e) => tracing::error!(%e, "Failed to pause playback"),
                },
                Ok(Some(uri)) if uri.is_empty() => {
                    tracing::info!("Read empty tag");
                }
                Ok(Some(uri)) => match start_playback(&mut client, device.id.clone(), uri.clone(), &self.observer) {
                    Ok(_) => tracing::info!(%uri, "Started playback"),
                    Err(e) => tracing::error!(%e, %uri, "Failed to start playback"),
                },
                Err(e) => {
                    tracing::warn!(%e, "Failed to read the URI from the card");
                }
            }
        }
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

fn pause_playback<O>(client: &mut spotify::Client, observer: &O) -> anyhow::Result<()>
where
    O: Observer,
{
    // Song may not be playing.
    if let Err(e) = client.pause(None) {
        if e.status() == Some(reqwest::StatusCode::FORBIDDEN) {
            return Ok(());
        }
    };

    observer.on_playback_paused();

    Ok(())
}

fn start_playback<O>(
    client: &mut spotify::Client,
    device_id: String,
    uri: String,
    observer: &O,
) -> anyhow::Result<()> where
    O: Observer,
{
    let uri: spotify::Uri = uri.as_str().parse()?;
    let mut uris = Vec::new();

    let queue = client.get_queue()?;
    let tracks: HashSet<String> = queue.currently_playing.into_iter()
        .chain(queue.queue.into_iter())
        .map(|i| i.uri)
        .collect();

    match uri.category.as_str() {
        "track" => {
            let track = client.get_track(&uri.id)?;
            uris.push(uri.to_string());
            observer.on_playback_started(Playable::Track(track));
        }
        "playlist" => {
            let playlist = client.get_playlist(&uri.id)?;
            uris.reserve(playlist.tracks.items.len());
            for item in playlist.tracks.items.iter() {
                uris.push(item.track.uri.clone());
            }
            observer.on_playback_started(Playable::Playlist(playlist));
        }
        "album" => {
            let album = client.get_album(&uri.id)?;
            if let Some(tracks) = &album.tracks {
                uris.reserve(tracks.items.len());
                for item in tracks.items.iter() {
                    uris.push(item.uri.clone());
                }
            }
            observer.on_playback_started(Playable::Album(album));
        }
        _ => {
            return Err(anyhow!("Unsupported URI category"));
        }
    }

    if !tracks.is_empty() && tracks.iter().all(|t| uris.contains(t)) {
        Ok(client.skip_to_next(None)?)
    } else {
        uris.shuffle(&mut rand::thread_rng());
        let request = StartPlaybackRequest::from(uris);
        Ok(client.play(Some(device_id), &request)?)
    }
}
