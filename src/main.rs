use crate::spotify::models::{Device, StartPlaybackRequest};
use anyhow::anyhow;

pub mod card;
pub mod spotify;
pub mod token;

use clap::Parser;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

static SLEEP_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Debug, Parser)]
#[command(author, version, about, long_about)]
struct Arguments {
    #[arg(short = 'v', long = None, action = clap::ArgAction::Count)]
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

    set_log_level(&arguments);

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

fn set_log_level(arguments: &Arguments) {
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

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");
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
    let mut request = StartPlaybackRequest::default();
    let uri: spotify::Uri = uri.as_str().parse()?;

    match uri.category.as_str() {
        "track" => {
            request.uris = Some(vec![uri.to_string()]);
        }
        "playlist" => {
            request.context_uri = Some(uri.to_string());
        }
        "album" => {
            request.context_uri = Some(uri.to_string());
        }
        _ => {
            return Err(anyhow!("Unsupported URI category"));
        }
    }

    client.play(device_id, &request)?;

    // Sometimes shuffle is unable to find a playback session.
    if let Err(e) = client.shuffle(true) {
        if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
            client.shuffle(true)?;
        }
    };

    Ok(())
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
