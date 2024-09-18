use anyhow::anyhow;
use clap::Parser;
use jukebox::{spotify, token};
use slint::SharedString;
use std::fmt::Write as _;
use std::path::PathBuf;

slint::slint! {
import { Button, GridBox, LineEdit } from "std-widgets.slint";

export component AppWindow inherits Window {
    preferred-width: 640px;
    preferred-height: 400px;
    icon: @image-url("assets/jukebox.png");
    title: @tr("Jukebox Tagger");

    in property <string> result: "";
    in-out property <string> value: "";

    callback read-value();
    callback write-value();
    callback describe();

    GridBox {
        Row {
            LineEdit {
                colspan: 3;
                placeholder-text: "Enter value here";
                edited => { root.value = self.text; }
            }
        }

        Row {
            Text {
                colspan: 3;
                text: root.result;
            }
        }

        Row {
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

    #[arg(short, long, env = "JUKEBOX_MARKET")]
    market: String,
}

fn main() -> Result<(), slint::PlatformError> {
    let arguments = Arguments::parse();
    let ui = AppWindow::new()?;

    ui.on_read_value({
        let ui_handle = ui.as_weak();

        move || {
            if let Some(ui) = ui_handle.upgrade() {
                match read_value() {
                    Ok(value) => {
                        ui.set_value(SharedString::from(value));
                        ui.set_result(SharedString::from(String::new()));
                    }
                    Err(e) => ui.set_result(slint::format!("Error: {}", e)),
                }
            }
        }
    });

    ui.on_write_value({
        let ui_handle = ui.as_weak();

        move || {
            if let Some(ui) = ui_handle.upgrade() {
                match write_value(ui.get_value().as_str().to_string()) {
                    Ok(_) => {
                        ui.set_result(slint::format!("Done"));
                    }
                    Err(e) => ui.set_result(slint::format!("Error: {}", e)),
                }
            }
        }
    });

    ui.on_describe({
        let ui_handle = ui.as_weak();

        move || {
            if let Some(ui) = ui_handle.upgrade() {
                let client_id = arguments.client_id.clone();
                let token_cache = arguments.token_cache.clone();
                let market = arguments.market.clone();
                let value = ui.get_value();

                match describe(client_id, token_cache, market, value.as_str()) {
                    Ok(description) => {
                        ui.set_result(SharedString::from(description));
                    }
                    Err(e) => ui.set_result(slint::format!("Error: {}", e)),
                }
            }
        }
    });

    ui.run()
}

fn read_value() -> anyhow::Result<String> {
    let ctx = pcsc::Context::establish(pcsc::Scope::User)?;
    let reader = jukebox::choose_reader(ctx, false)?;

    match reader.read()? {
        None => Err(anyhow!("No card is present.")),
        Some(value) => Ok(value),
    }
}

fn write_value(value: String) -> anyhow::Result<()> {
    let ctx = pcsc::Context::establish(pcsc::Scope::User)?;
    let reader = jukebox::choose_reader(ctx, false)?;

    if reader.write(value)? {
        return Err(anyhow!("No card is present."));
    } else {
        Ok(())
    }
}

fn describe(
    client_id: String,
    token_cache: PathBuf,
    market: String,
    value: &str,
) -> anyhow::Result<String> {
    let oauth = token::Client::new(client_id, token_cache);
    let mut client = spotify::Client::new(oauth, market);
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

            write!(&mut description, "Tracks: {}\n", album.total_tracks)?;

            Ok(description)
        }
        _ => Err(anyhow!("Unsupported URI category")),
    }
}
