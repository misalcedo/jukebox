use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about)]
pub struct Arguments {
    #[arg(short = 'v', long = None, env = "JUKEBOX_VERBOSITY", action = clap::ArgAction::Count)]
    pub verbosity: u8,

    #[arg(short, long, env = "JUKEBOX_CLIENT_ID")]
    pub client_id: String,

    #[arg(short, long, env = "JUKEBOX_TOKEN_CACHE")]
    pub token_cache: PathBuf,

    #[arg(short, long, env = "JUKEBOX_MARKET")]
    pub market: String,

    #[arg(short, long, env = "JUKEBOX_DEVICE")]
    pub device: Option<String>,

    #[arg(short, long, env = "JUKEBOX_ADDRESS")]
    pub address: String,
}
