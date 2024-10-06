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

    if let Err(e) = run(arguments) {
        tracing::error!(%e, "Unable to run the jukebox");
    }
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
fn run(arguments: cli::Arguments) -> anyhow::Result<()> {
    let window = app::Window::new()?;
    let player = Player::from(window.observer());

    let join_handle = thread::spawn(move || {
        loop {
            match player.run(&arguments) {
                Ok(_) => break,
                Err(e) => tracing::warn!(%e, "Restarting the player"),
            }

            thread::sleep(SLEEP_INTERVAL);
        }
    });

    window.run()?;

    if let Err(_) = join_handle.join() {
        return Err(anyhow!("Failed to join the player thread"));
    }

    Ok(())
}

#[cfg(not(feature = "ui"))]
fn run(arguments: cli::Arguments) -> anyhow::Result<()> {
    let player = Player::default();

    loop {
        match player.run(&arguments) {
            Ok(_) => break,
            Err(e) => tracing::warn!(%e, "Restarting the player"),
        }

        thread::sleep(SLEEP_INTERVAL);
    }

    Ok(())
}
