use anyhow::anyhow;
use clap::{Parser, Subcommand};
use jukebox::spotify::models::{Device, StartPlaybackRequest};
use jukebox::{card, spotify, token};
use std::path::PathBuf;
use url::Url;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about)]
struct Arguments {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Groove(Groove),
    Write(Write),
    Erase(Erase),
    Read(Read),
}

#[derive(Debug, Parser)]
struct Groove {
    #[arg(short, long, env = "JUKEBOX_CLIENT_ID")]
    client_id: String,

    #[arg(short, long, env = "JUKEBOX_TOKEN_CACHE")]
    token_cache: PathBuf,

    #[arg(short, long, default_value = "Miguel’s MacBook Pro (2)")]
    device: Option<String>,
}

#[derive(Debug, Parser)]
struct Write {
    #[arg(short, long)]
    uri: Url,
}

#[derive(Debug, Parser)]
struct Erase {}

#[derive(Debug, Parser)]
struct Read {}

fn main() {
    let arguments = Arguments::parse();

    match arguments.command {
        Commands::Groove(groove) => {
            let oauth = token::Client::new(groove.client_id, groove.token_cache);
            let mut client = spotify::Client::new(oauth);

            let device = choose_device(&mut client, groove.device.as_deref())
                .expect("Failed to choose a device.");

            let ctx = pcsc::Context::establish(pcsc::Scope::User).expect("Failed to establish context");
            let reader = choose_reader(ctx).expect("Failed to choose a card reader.");

            loop {
                reader.wait(None).expect("Failed to wait for a card to be present.");

                match reader.read().expect("Failed to read the URI from the card.") {
                    None => {
                        eprintln!("No card is present.");
                    }
                    Some(uri) => {
                        if let Err(error) =
                            start_playback(&mut client, device.id.clone(), uri.to_string())
                        {
                            eprintln!("Failed to start playback: {}", error);
                        }
                    }
                }
            }
        }
        Commands::Write(write) => {
            let uri = spotify::normalize_uri(&write.uri).expect("Failed to normalize the track URI");
            let ctx = pcsc::Context::establish(pcsc::Scope::User).expect("Failed to establish context");
            let reader = choose_reader(ctx).expect("Failed to choose a card reader.");

            if !reader.write(uri).expect("Failed to write the URI to the card.") {
                eprintln!("No card is present.");
            }
        }
        Commands::Erase(_) => {
            let ctx = pcsc::Context::establish(pcsc::Scope::User).expect("Failed to establish context");
            let reader = choose_reader(ctx).expect("Failed to choose a card reader.");

            if !reader.erase().expect("Failed to erase the card.") {
                eprintln!("No card is present.");
            }
        }
        Commands::Read(_) => {
            let ctx = pcsc::Context::establish(pcsc::Scope::User).expect("Failed to establish context");
            let reader = choose_reader(ctx).expect("Failed to choose a card reader.");

            match reader.read() {
                Ok(None) => {
                    eprintln!("No card is present.");
                }
                Ok(Some(value)) => {
                    println!("{value:?}");
                }
                Err(err) => {
                    eprintln!("Failed to read the URI from the card: {}", err);
                }
            }
        }
    }
}

fn choose_reader(ctx: pcsc::Context) -> anyhow::Result<card::Reader> {
    let mut readers = ctx.list_readers_owned()?;
    // Look for "ACS ACR1252 1S CL Reader PICC 0"
    let reader = readers.pop().ok_or_else(|| anyhow!("No readers are connected."))?;

    Ok(card::Reader::new(ctx, reader))
}

fn choose_device(client: &mut spotify::Client, name: Option<&str>) -> anyhow::Result<Device> {
    let device = client
        .get_available_devices()?
        .devices
        .into_iter()
        .find(|device| match name {
            None => true,
            Some(name) => &device.name == name,
        })
        .ok_or_else(|| anyhow!("Found no matching device"))?;

    Ok(device)
}

fn start_playback(
    client: &mut spotify::Client,
    device_id: String,
    uri: String,
) -> anyhow::Result<()> {
    let mut request = StartPlaybackRequest::default();

    let (category, _) =
        spotify::uri_parts(&uri).ok_or_else(|| anyhow!("Failed to extract category from URI"))?;
    match category {
        "track" => {
            request.uris = Some(vec![uri]);
        }
        "playlist" => {
            request.context_uri = Some(uri);
        }
        "album" => {
            request.context_uri = Some(uri);
        }
        _ => {
            return Err(anyhow!("Unsupported URI category"));
        }
    }

    client.play(Some(device_id), &request)?;

    // Sometimes shuffle is unable to find a playback session.
    if let Err(err) = client.shuffle(true) {
        if err.status() == Some(reqwest::StatusCode::NOT_FOUND) {
            client.shuffle(true)?;
        }
    };

    Ok(())
}
