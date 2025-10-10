mod card;
mod cli;
mod console;
mod local;
mod player;
mod spotify;
mod token;
mod web;
mod progress;

use crate::card::Reader;
use crate::cli::Arguments;
use crate::console::Screen;
use clap::Parser;
use std::io;
use tokio::sync::watch::Sender;
use tracing_log::LogTracer;

fn main() {
    let arguments = cli::Arguments::parse();
    let screen = set_log_level(&arguments).expect("Failed to configure logging");

    tracing::debug!(?arguments, "starting jukebox server");

    if let Err(e) = run(arguments, screen) {
        tracing::error!(%e, "Unable to run the jukebox");
    }
}

fn set_log_level(arguments: &Arguments) -> anyhow::Result<console::Screen> {
    LogTracer::init()?;

    let level = match arguments.verbosity {
        0 => tracing::Level::ERROR,
        1 => tracing::Level::WARN,
        2 => tracing::Level::INFO,
        3 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    let screen = Screen::default();
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_writer(tracing_subscriber::fmt::writer::Tee::new(
            io::stderr,
            screen.clone(),
        ))
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(screen)
}

fn run(arguments: Arguments, screen: Screen) -> anyhow::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()?;
    let result: anyhow::Result<()> = runtime.block_on(async {
        let (sender, receiver) = tokio::sync::watch::channel(None);

        let mut group = tokio::task::JoinSet::new();
        let oauth = token::Client::new(arguments.client_id, arguments.token_cache);
        let client = spotify::Client::new(oauth.clone(), arguments.market);
        let stream_player = spotify::Player::new(client.clone(), arguments.device);
        let file_player = local::Player::new(arguments.local_music_path);

        // Construct a local task set that can run `!Send` futures.
        let local = tokio::task::LocalSet::new();

        group.spawn(web::run(
            sender.clone(),
            receiver.clone(),
            oauth,
            arguments.address,
            screen,
            client.clone(),
        ));

        group.spawn_local_on(player::run(receiver, stream_player, file_player), &local);
        group.spawn_blocking(|| read_loop(sender));

        while let Some(join_result) = local.run_until(group.join_next()).await {
            // TODO: uncomment to fail without NFC reader
            // join_result??
        }

        Ok(())
    });

    if let Err(e) = result {
        tracing::error!(%e, "Failed to run the jukebox");
    }

    Ok(())
}

fn read_loop(sender: Sender<Option<String>>) -> anyhow::Result<()> {
    let ctx = pcsc::Context::establish(pcsc::Scope::User)?;
    let mut reader = Reader::try_from(ctx)?;

    tracing::debug!("Waiting for a card to be inserted");

    loop {
        reader.wait(None)?;
        match reader.read() {
            Ok(card) => {
                tracing::debug!(?card, "Read a card");
                sender.send(card)?;
            }
            Err(e) => {
                tracing::warn!(%e, "Failed to read the URI from the card");
                sender.send(None)?;
            }
        }
    }
}
