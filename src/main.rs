mod card;
mod cli;
mod player;
mod spotify;
mod token;
mod web;

use crate::card::Reader;
use crate::cli::Arguments;
use clap::Parser;
use tokio::sync::watch::Sender;
use tracing_log::LogTracer;

fn main() {
    let arguments = cli::Arguments::parse();

    if let Err(e) = set_log_level(&arguments) {
        eprintln!("Failed to configure logging: {e}");
    };

    if let Err(e) = run(arguments) {
        tracing::error!(%e, "Unable to run the jukebox");
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

fn run(arguments: Arguments) -> anyhow::Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    let results = runtime.block_on(async {
        let (sender, receiver) = tokio::sync::watch::channel(None);

        let mut group = tokio::task::JoinSet::new();
        let oauth = token::Client::new(arguments.client_id, arguments.token_cache);

        group.spawn(web::run(
            sender.clone(),
            receiver.clone(),
            oauth.clone(),
            arguments.address,
        ));
        group.spawn(player::run(
            receiver,
            oauth,
            arguments.market,
            arguments.device,
        ));
        group.spawn_blocking(|| read_loop(sender));
        group.join_all().await
    });

    for result in results {
        if let Err(e) = result {
            tracing::error!(%e, "Failed to run the jukebox");
        }
    }

    Ok(())
}

fn read_loop(sender: Sender<Option<String>>) -> anyhow::Result<()> {
    let ctx = pcsc::Context::establish(pcsc::Scope::User)?;
    let mut reader = Reader::try_from(ctx)?;

    loop {
        reader.wait(None)?;
        match reader.read() {
            Ok(card) => {
                sender.send(card)?;
            }
            Err(e) => {
                tracing::warn!(%e, "Failed to read the URI from the card");
                sender.send(None)?;
            }
        }
    }
}
