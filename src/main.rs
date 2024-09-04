use crate::spotify::models::StartPlaybackRequest;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use url::Url;
use crate::spotify::{normalize_uri, uri_parts};

mod spotify;
mod token;

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

    #[arg(short, long, default_value = "spotify:track:6b2HYgqcK9mvktt4GxAu72", value_parser = Url::parse)]
    uri: Url,
}

#[derive(Debug, Parser)]
struct Write {
    #[arg(short, long)]
    uri: Url,
}

#[derive(Debug, Parser)]
struct Read {}

fn main() {
    let arguments = Arguments::parse();

    match arguments.command {
        Commands::Groove(groove) => {
            let oauth = token::Client::new(groove.client_id, groove.token_cache);
            let mut client = spotify::Client::new(oauth);

            let device = client
                .get_available_devices()
                .unwrap_or_default()
                .devices
                .into_iter()
                .find(|device| match &groove.device {
                    None => true,
                    Some(name) => &device.name == name,
                })
                .expect("Failed to find a device.");

            let mut request = StartPlaybackRequest {
                context_uri: None,
                offset: None,
                uris: None,
                position_ms: 0,
            };

            let uri = normalize_uri(&groove.uri).expect("Failed to normalize the track URI");
            let (category, _) = uri_parts(&uri).expect("Failed to extract URI parts");

            match category{
                "track" => {
                    request.uris = Some(vec![uri]);
                }
                "playlist" => {
                    request.context_uri = Some(uri);
                }
                "album" => {
                    request.context_uri = Some(uri);
                }
                part => {
                    println!("Unsupported URI: {:?} ({:?})", groove.uri, part);
                }
            }

            client.play(Some(device.id), &request).expect("Failed to play");
            client.shuffle(true).expect("Failed to shuffle");
        }
        Commands::Write(write) => {
            let uri =
                normalize_uri(&write.uri).expect("Failed to normalize the track URI");

            println!("{}", uri);
        }
        Commands::Read(_) => {}
    }
}
