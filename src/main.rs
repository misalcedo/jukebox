use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};
use crate::spotify::models::{SearchRequest, StartPlaybackRequest};

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
    Search(Search)
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
    value: String,
}

#[derive(Debug, Parser)]
struct Read {
}

#[derive(Debug, Parser)]
struct Search {
    #[arg(short, long, env = "JUKEBOX_CLIENT_ID")]
    client_id: String,

    #[arg(short, long, env = "JUKEBOX_TOKEN_CACHE")]
    token_cache: PathBuf,

    #[arg(short, long)]
    query: String,

    #[arg(short, long)]
    kind: SearchKind,

    #[arg(short, long, default_value_t = 0)]
    offset: usize
}

#[derive(Clone, Debug, ValueEnum, Default)]
enum SearchKind {
    Album,
    Artist,
    Playlist,
    #[default]
    Track,
    Show,
    Episode,
    Audiobook
}

impl Display for SearchKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Album => write!(f, "album"),
            Self::Artist => write!(f, "artist"),
            Self::Playlist => write!(f, "playlist"),
            Self::Track => write!(f, "track"),
            Self::Show => write!(f, "show"),
            Self::Episode => write!(f, "episode"),
            Self::Audiobook => write!(f, "audiobook"),
        }
    }
}

fn main() {
    let arguments = Arguments::parse();

    match arguments.command {
        Commands::Groove(groove) => {
            let oauth = token::Client::new(groove.client_id, groove.token_cache);
            let mut client = spotify::Client::new(oauth);

            let device = client.get_available_devices().unwrap_or_default().devices.into_iter().find(|device| {
                match &groove.device {
                    None => true,
                    Some(name) => &device.name == name
                }
            }).expect("Failed to find a device.");

            client.play(Some(device.id), &StartPlaybackRequest {
                context_uri: None,
                offset: None,
                uris: vec!["spotify:track:6b2HYgqcK9mvktt4GxAu72".to_string()],
                position_ms: 0,
            }).expect("Failed to play");
        }
        Commands::Write(_) => {}
        Commands::Read(_) => {}
        Commands::Search(search) => {
            let oauth = token::Client::new(search.client_id, search.token_cache);
            let mut client = spotify::Client::new(oauth);


            client.search(&SearchRequest {
                q: search.query,
                r#type: search.kind.to_string(),
                offset: search.offset.to_string(),
            }).expect("Failed to play");
        }
    }

}
