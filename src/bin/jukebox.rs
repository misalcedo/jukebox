use clap::{Parser, Subcommand};
use jukebox::{spotify, token};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about)]
struct Arguments {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Login(Login),
    Groove(Groove),
    Write(Write),
    Erase(Erase),
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

    #[arg(short, long, default_value = "Miguel’s MacBook Pro (2)")]
    device: Option<String>,
}

#[derive(Debug, Parser)]
struct Write {
    #[arg(short, long)]
    uri: spotify::Uri,
}

#[derive(Debug, Parser)]
struct Erase {}

#[derive(Debug, Parser)]
struct Read {}

fn main() {
    let arguments = Arguments::parse();

    match arguments.command {
        Commands::Login(login) => {
            let mut oauth = token::Client::new(login.client_id, login.token_cache);
            oauth.authorization();
        }
        Commands::Groove(groove) => {
            let oauth = token::Client::new(groove.client_id, groove.token_cache);
            let mut client = spotify::Client::new(oauth);

            let device = jukebox::choose_device(&mut client, groove.device.as_deref())
                .expect("Failed to choose a device.");

            let ctx =
                pcsc::Context::establish(pcsc::Scope::User).expect("Failed to establish context");
            let reader = jukebox::choose_reader(ctx).expect("Failed to choose a card reader.");

            loop {
                reader
                    .wait(None)
                    .expect("Failed to wait for a card to be present.");

                match reader
                    .read()
                    .expect("Failed to read the URI from the card.")
                {
                    None => {
                        eprintln!("No card is present.");
                    }
                    Some(uri) => {
                        if let Err(error) =
                            jukebox::start_playback(&mut client, device.id.clone(), uri)
                        {
                            eprintln!("Failed to start playback: {}", error);
                        }
                    }
                }
            }
        }
        Commands::Write(write) => {
            let ctx =
                pcsc::Context::establish(pcsc::Scope::User).expect("Failed to establish context");
            let reader = jukebox::choose_reader(ctx).expect("Failed to choose a card reader.");

            if !reader
                .write(write.uri.to_string())
                .expect("Failed to write the URI to the card.")
            {
                eprintln!("No card is present.");
            }
        }
        Commands::Erase(_) => {
            let ctx =
                pcsc::Context::establish(pcsc::Scope::User).expect("Failed to establish context");
            let reader = jukebox::choose_reader(ctx).expect("Failed to choose a card reader.");

            if !reader.erase().expect("Failed to erase the card.") {
                eprintln!("No card is present.");
            }
        }
        Commands::Read(_) => {
            let ctx =
                pcsc::Context::establish(pcsc::Scope::User).expect("Failed to establish context");
            let reader = jukebox::choose_reader(ctx).expect("Failed to choose a card reader.");

            match reader.read() {
                Ok(None) => {
                    eprintln!("No card is present.");
                }
                Ok(Some(value)) => {
                    println!("{value:?}");
                }
                Err(e) => {
                    eprintln!("Failed to read the URI from the card: {}", e);
                }
            }
        }
    }
}
