use anyhow::anyhow;
use clap::Parser;
use jukebox::{spotify, token};
use slint::SharedString;
use std::fmt::Write as _;
use std::path::PathBuf;

slint::slint! {
import { Button, HorizontalBox, LineEdit, VerticalBox } from "std-widgets.slint";

export component AppWindow inherits Window {
    width: 640px;
    height: 400px;

    in property <string> error: "";
    in-out property <string> value: "";

    callback read-value();
    callback write-value();
    callback describe();

    VerticalBox {
        LineEdit {
            placeholder-text: "Enter value here";
            text: root.value;
        }
        Text {
            text: root.error;
        }
        HorizontalBox {
            Button {
                text: "Read";
                clicked => {
                    root.read-value();
                }
            }
            Button {
                text: "Write";
                clicked => {
                    root.write-value();
                }
            }
            Button {
                text: "Describe";
                clicked => {
                    root.describe();
                }
            }
        }
    }
}
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about)]
struct Arguments {
    #[arg(short, long, env = "JUKEBOX_CLIENT_ID")]
    client_id: String,

    #[arg(short, long, env = "JUKEBOX_TOKEN_CACHE")]
    token_cache: PathBuf,
}

fn main() -> Result<(), slint::PlatformError> {
    let arguments = Arguments::parse();
    let ui = AppWindow::new()?;

    ui.on_read_value({
        let ui_handle = ui.as_weak();

        move || {
            if let Some(ui) = ui_handle.upgrade() {
                let value = read_value().expect("Failed to read the URI from the card.");
                ui.set_value(SharedString::from(value));
            }
        }
    });

    ui.on_write_value({
        let ui_handle = ui.as_weak();

        move || {
            if let Some(ui) = ui_handle.upgrade() {
                write_value(ui.get_value().as_str().to_string())
                    .expect("Failed to write the URI to the card.");
            }
        }
    });

    ui.on_describe({
        let ui_handle = ui.as_weak();

        move || {
            if let Some(ui) = ui_handle.upgrade() {
                let client_id = arguments.client_id.clone();
                let token_cache = arguments.token_cache.clone();
                let value = ui.get_value();

                match describe(client_id, token_cache, value.as_str()) {
                    Ok(_) => {}
                    Err(e) => {}
                }
            }
        }
    });

    ui.run()
}

fn read_value() -> anyhow::Result<String> {
    let ctx = pcsc::Context::establish(pcsc::Scope::User)?;
    let reader = jukebox::choose_reader(ctx)?;

    match reader.read()? {
        None => Err(anyhow!("No card is present.")),
        Some(value) => Ok(value)
    }
}

fn write_value(value: String) -> anyhow::Result<()> {
    let ctx = pcsc::Context::establish(pcsc::Scope::User)?;
    let reader = jukebox::choose_reader(ctx)?;

    if reader.write(value)? {
        return Err(anyhow!("No card is present."));
    } else {
        Ok(())
    }
}

fn describe(client_id: String, token_cache: PathBuf, value: &str) -> anyhow::Result<String> {
    let oauth = token::Client::new(client_id, token_cache);
    let mut client = spotify::Client::new(oauth);
    let uri: spotify::Uri = value.parse()?;

    match uri.category.as_str() {
        "track" => {
            let track = client.get_track(&uri.id)?;
            let mut description = String::new();

            write!(&mut description, "Track: {}\n", track.name)?;
            write!(&mut description, "Album: {}\n", track.album.name)?;

            if !track.artists.is_empty() {
                let artists: Vec<&str> = track.artists.iter().map(|a| a.name.as_str()).collect();
                write!(&mut description, "Artists: {}\n", artists.join(", "))?;
            }

            Ok(description)
        }
        "playlist" => {
            let playlist = client.get_playlist(&uri.id)?;
            let mut description = String::new();

            write!(&mut description, "Playlist: {}\n", playlist.name)?;
            write!(&mut description, "Owner: {}\n", playlist.owner.display_name)?;

            Ok(description)
        }
        "album" => {
            let album = client.get_album(&uri.id)?;
            let mut description = String::new();

            write!(&mut description, "Album: {}\n", album.name)?;

            if !album.artists.is_empty() {
                let artists: Vec<&str> = album.artists.iter().map(|a| a.name.as_str()).collect();
                write!(&mut description, "Artists: {}\n", artists.join(", "))?;
            }

            Ok(description)
        }
        _ => {
            Err(anyhow!("Unsupported URI category"))
        }
    }
}