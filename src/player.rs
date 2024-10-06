use crate::spotify::models::{Album, Device, Playlist, StartPlaybackRequest, Track};
use crate::{card, cli, spotify, token};
use anyhow::anyhow;
use rand::prelude::SliceRandom;

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
    device_id: Option<String>,
    last_request: StartPlaybackRequest,
}

impl<O> From<O> for Player<O>
where
    O: Observer,
{
    fn from(observer: O) -> Self {
        Self {
            observer,
            device_id: None,
            last_request: StartPlaybackRequest::default(),
        }
    }
}

impl Default for Player<()> {
    fn default() -> Self {
        Self {
            observer: (),
            device_id: None,
            last_request: StartPlaybackRequest::default(),
        }
    }
}

impl<O> Player<O>
where
    O: Observer,
{
    pub fn run(&mut self, arguments: &cli::Arguments) -> anyhow::Result<()> {
        self.last_request = StartPlaybackRequest::default();

        let oauth = token::Client::new(arguments.client_id.clone(), arguments.token_cache.clone());
        let mut client = spotify::Client::new(oauth, arguments.market.clone());

        let device = choose_device(&mut client, arguments.device.as_deref())?;

        self.device_id = Some(device.id);

        let ctx = pcsc::Context::establish(pcsc::Scope::User)?;
        let mut reader = choose_reader(ctx)?;

        loop {
            reader.wait(None)?;

            match reader.read() {
                Ok(None) => match self.pause_playback(&mut client) {
                    Ok(_) => tracing::info!("Paused playback"),
                    Err(e) => tracing::error!(%e, "Failed to pause playback"),
                },
                Ok(Some(uri)) if uri.is_empty() => {
                    tracing::info!("Read empty tag");
                }
                Ok(Some(uri)) => match self.start_playback(&mut client, uri.clone()) {
                    Ok(_) => tracing::info!(%uri, "Started playback"),
                    Err(e) => tracing::error!(%e, %uri, "Failed to start playback"),
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

    fn start_playback(
        &mut self,
        client: &mut spotify::Client,
        uri: String,
    ) -> anyhow::Result<()> {
        let uri: spotify::Uri = uri.as_str().parse()?;
        let mut uris = Vec::new();

        match uri.category.as_str() {
            "track" => {
                let track = client.get_track(&uri.id)?;
                uris.push(uri.to_string());
                self.observer.on_playback_started(Playable::Track(track));
            }
            "playlist" => {
                let playlist = client.get_playlist(&uri.id)?;
                uris.reserve(playlist.tracks.items.len());
                for item in playlist.tracks.items.iter() {
                    uris.push(item.track.uri.clone());
                }
                self.observer.on_playback_started(Playable::Playlist(playlist));
            }
            "album" => {
                let album = client.get_album(&uri.id)?;
                if let Some(tracks) = &album.tracks {
                    uris.reserve(tracks.items.len());
                    for item in tracks.items.iter() {
                        uris.push(item.uri.clone());
                    }
                }
                self.observer.on_playback_started(Playable::Album(album));
            }
            _ => {
                return Err(anyhow!("Unsupported URI category"));
            }
        }

        if let Some(state) = client.get_playback_state()? {
            self.device_id = Some(state.device.id);

            if let Some(item) = state.item {
                if self.last_request.uris.contains(&item.uri) && self.last_request.uris.last() != Some(&item.uri) {
                    client.skip_to_next(None)?;
                    return Ok(());
                }
            }
        }

        uris.shuffle(&mut rand::thread_rng());
        self.last_request = StartPlaybackRequest::from(uris);
        client.play(self.device_id.clone(), &self.last_request)?;
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