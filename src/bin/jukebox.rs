use clap::{Parser, Subcommand};
use jukebox::spotify::{Uri, UriParseError};
use jukebox::{spotify, token};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about)]
struct Arguments {
    #[command(subcommand)]
    command: Commands,

    #[arg(short = 'v', long = None, global = true, action = clap::ArgAction::Count)]
    verbosity: u8,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Login(Login),
    Groove(Groove),
    Read(Read),
}

#[derive(Debug, Parser)]
struct Login {
    #[arg(short, long, env = "JUKEBOX_CLIENT_ID")]
    client_id: String,

    #[arg(short, long, env = "JUKEBOX_TOKEN_CACHE")]
    token_cache: PathBuf,
}

#[derive(Debug, Parser)]
struct Groove {
    #[arg(short, long, env = "JUKEBOX_CLIENT_ID")]
    client_id: String,

    #[arg(short, long, env = "JUKEBOX_TOKEN_CACHE")]
    token_cache: PathBuf,

    #[arg(short, long, env = "JUKEBOX_MARKET")]
    market: String,

    #[arg(short, long, env = "JUKEBOX_DEVICE")]
    device: Option<String>,
}

#[derive(Debug, Parser)]
struct Read {
    #[arg(short, long)]
    normalize: bool,
}

fn main() {
    let arguments = Arguments::parse();

    set_log_level(&arguments);

    match arguments.command {
        Commands::Login(login) => {
            let mut oauth = token::Client::new(login.client_id, login.token_cache);
            oauth.authorization();
        }
        Commands::Groove(groove) => {
            let oauth = token::Client::new(groove.client_id, groove.token_cache);
            let mut client = spotify::Client::new(oauth, groove.market);

            let device = jukebox::choose_device(&mut client, groove.device.as_deref())
                .expect("Failed to choose a device");

            let ctx =
                pcsc::Context::establish(pcsc::Scope::User).expect("Failed to establish context");
            let mut reader = jukebox::choose_reader(ctx).expect("Failed to choose a card reader");

            loop {
                reader
                    .wait(None)
                    .expect("Failed to wait for a card to be present");

                match reader.read() {
                    Ok(None) => {
                        match jukebox::pause_playback(&mut client, device.id.clone()) {
                            Ok(_) => tracing::debug!("Paused playback"),
                            Err(e) => tracing::error!(%e, "Failed to pause playback")
                        }
                    }
                    Ok(Some(uri)) if uri.is_empty() => {
                        tracing::debug!("Read empty tag");
                    }
                    Ok(Some(uri)) => {
                        match jukebox::start_playback(&mut client, device.id.clone(), uri.clone()) {
                            Ok(_) => tracing::debug!(%uri, "Started playback"),
                            Err(e) => tracing::error!(%e, %uri, "Failed to start playback")
                        }
                    }
                    Err(e) => {
                        tracing::error!(%e, "Failed to read the URI from the card");
                    }
                }
            }
        }
        Commands::Read(read) => {
            let ctx =
                pcsc::Context::establish(pcsc::Scope::User).expect("Failed to establish context");
            let reader = jukebox::choose_reader(ctx).expect("Failed to choose a card reader");

            match reader.read() {
                Ok(None) => {
                    tracing::warn!("No card is present");
                }
                Ok(Some(value)) => {
                    if read.normalize {
                        let result: Result<Uri, UriParseError> = value.as_str().parse();
                        match result {
                            Ok(uri) => println!("{:?}", uri.to_string()),
                            Err(_) => {
                                tracing::warn!("Failed to parse the URI");
                                println!("{value:?}");
                            }
                        }
                    } else {
                        println!("{value:?}");
                    }
                }
                Err(e) => {
                    tracing::error!(%e, "Failed to read the URI from the card");
                }
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

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global default subscriber");
}
