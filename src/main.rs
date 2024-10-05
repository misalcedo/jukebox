mod card;
mod cli;
mod player;
mod spotify;
mod token;

#[cfg(feature = "ui")]
mod app;

use crate::player::Player;
use clap::Parser;
use std::thread;
use std::time::Duration;
use tracing_log::LogTracer;

static SLEEP_INTERVAL: Duration = Duration::from_secs(1);

fn main() {
    let arguments = cli::Arguments::parse();

    if let Err(e) = set_log_level(&arguments) {
        eprintln!("Failed to configure logging: {e}");
    };

    run(arguments);
}

fn set_log_level(arguments: &cli::Arguments) -> anyhow::Result<()> {
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


#[cfg(feature = "ui")]
fn run(arguments: cli::Arguments) {
    let mut player = Player::default();

    let join_handle = thread::spawn(move || {
        loop {
            match player.run(&arguments) {
                Ok(_) => (),
                Err(e) => tracing::warn!(%e, "Restarting the player"),
            }

            thread::sleep(SLEEP_INTERVAL);
        }
    });

    app::run().expect("Failed to run the UI");

    join_handle.join().expect("Failed to join the player thread");
}

#[cfg(not(feature = "ui"))]
fn run(arguments: cli::Arguments) {
    let player = Player::default();

    loop {
        match player.run(&arguments) {
            Ok(_) => (),
            Err(e) => tracing::warn!(%e, "Restarting the player"),
        }

        thread::sleep(SLEEP_INTERVAL);
    }
}
