use crate::spotify::models::{Device, StartPlaybackRequest};
use anyhow::anyhow;

pub mod card;
pub mod spotify;
pub mod token;

#[cfg(feature = "ui")]
mod window;

use clap::Parser;
use rand::prelude::SliceRandom;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tracing_log::LogTracer;

static SLEEP_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Debug, Parser)]
#[command(author, version, about, long_about)]
struct Arguments {
    #[arg(short = 'v', long = None, env = "JUKEBOX_VERBOSITY", action = clap::ArgAction::Count)]
    verbosity: u8,

    #[arg(short, long, env = "JUKEBOX_CLIENT_ID")]
    client_id: String,

    #[arg(short, long, env = "JUKEBOX_TOKEN_CACHE")]
    token_cache: PathBuf,

    #[arg(short, long, env = "JUKEBOX_MARKET")]
    market: String,

    #[arg(short, long, env = "JUKEBOX_DEVICE")]
    device: String,
}

fn main() {
    let arguments = Arguments::parse();

    set_log_level(&arguments).expect("Failed to configure logging");

    let oauth = token::Client::new(arguments.client_id, arguments.token_cache);
    let mut client = spotify::Client::new(oauth, arguments.market);

    let device = choose_device(&mut client, &arguments.device).expect("Failed to choose a device");

    let ctx = pcsc::Context::establish(pcsc::Scope::User).expect("Failed to establish context");
    let mut reader = choose_reader(ctx).expect("Failed to choose a card reader");

    loop {
        reader
            .wait(None)
            .expect("Failed to wait for a card to be present");

        match reader.read() {
            Ok(None) => match pause_playback(&mut client, device.id.clone()) {
                Ok(_) => tracing::info!("Paused playback"),
                Err(e) => tracing::error!(%e, "Failed to pause playback"),
            },
            Ok(Some(uri)) if uri.is_empty() => {
                tracing::info!("Read empty tag");
            }
            Ok(Some(uri)) => match start_playback(&mut client, device.id.clone(), uri.clone()) {
                Ok(_) => tracing::info!(%uri, "Started playback"),
                Err(e) => tracing::error!(%e, %uri, "Failed to start playback"),
            },
            Err(e) => {
                tracing::error!(%e, "Failed to read the URI from the card");
            }
        }
    }
}

fn set_log_level(arguments: &Arguments) -> anyhow::Result<()> {
    LogTracer::init()?;

    let level = match arguments.verbosity {
        0 => tracing::Level::ERROR,
        1 => tracing::Level::WARN,
        2 => tracing::Level::INFO,
        3 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

pub fn choose_reader(ctx: pcsc::Context) -> anyhow::Result<card::Reader> {
    loop {
        for reader in ctx.list_readers_owned()? {
            if let Ok(name) = reader.to_str() {
                if name.contains("PICC") {
                    return Ok(card::Reader::new(ctx, reader));
                }
            }
        }

        tracing::warn!("No PICC readers are connected");
        thread::sleep(SLEEP_INTERVAL);
    }
}

pub fn choose_device(client: &mut spotify::Client, name: &str) -> anyhow::Result<Device> {
    loop {
        match client
            .get_available_devices()?
            .devices
            .into_iter()
            .find(|device| device.name == name) {
            None => {
                tracing::warn!("Found no matching device");
                thread::sleep(SLEEP_INTERVAL);
            }
            Some(device) => return Ok(device)
        }
    }
}

pub fn start_playback(
    client: &mut spotify::Client,
    device_id: String,
    uri: String,
) -> anyhow::Result<()> {
    let uri: spotify::Uri = uri.as_str().parse()?;
    let mut uris = Vec::new();
    let mut device = device_id;

    if let Some(state) = client.get_playback_state()? {
        device = state.device.id;
        if !state.is_playing && state.progress_ms < state.item.duration_ms {
            tracing::debug!("{state:?}");
        }
    }

    match uri.category.as_str() {
        "track" => {
            uris.push(uri.to_string());
        }
        "playlist" => {
            let playlist = client.get_playlist(&uri.id)?;
            uris.reserve(playlist.tracks.items.len());
            for item in playlist.tracks.items {
                uris.push(item.track.uri);
            }
        }
        "album" => {
            let album = client.get_album(&uri.id)?;
            if let Some(tracks) = album.tracks {
                uris.reserve(tracks.items.len());
                for item in tracks.items {
                    uris.push(item.uri);
                }
            }
        }
        _ => {
            return Err(anyhow!("Unsupported URI category"));
        }
    }

    uris.shuffle(&mut rand::thread_rng());

    let request = StartPlaybackRequest::from(uris);

    Ok(client.play(Some(device), &request)?)
}

pub fn pause_playback(client: &mut spotify::Client, device_id: String) -> anyhow::Result<()> {
    // Song may not be playing.
    if let Err(e) = client.pause(device_id) {
        if e.status() == Some(reqwest::StatusCode::FORBIDDEN) {
            return Ok(());
        }
    };

    Ok(())
}
