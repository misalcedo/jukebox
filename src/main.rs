use crate::spotify::models::StartPlaybackRequest;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use url::Url;

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

    #[arg(short, long)]
    device: Option<String>,
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

            client
                .play(
                    Some(device.id),
                    &StartPlaybackRequest {
                        context_uri: None,
                        offset: None,
                        uris: vec!["spotify:track:6b2HYgqcK9mvktt4GxAu72".to_string()],
                        position_ms: 0,
                    },
                )
                .expect("Failed to play");
        }
        Commands::Write(write) => {
            let uri =
                spotify::normalize_track(&write.uri).expect("Failed to normalize the track URI");

            println!("{}", uri);
        }
        Commands::Read(_) => {}
    }
}
