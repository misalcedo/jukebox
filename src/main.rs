slint::include_modules!();

use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Track {
    id: String,
    name: String,
    artists: Vec<Artist>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Artist {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResult {
    tracks: Tracks,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tracks {
    items: Vec<Track>,
}

async fn search_tracks(query: &str) -> Result<SearchResult, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("https://api.spotify.com/v1/search?q={}&type=track", query);

    let response = client.get(url).send().await?;
    let search_result: SearchResult = response.json().await?;

    Ok(search_result)
}

fn main() -> Result<(), slint::PlatformError>  {
    let ui = AppWindow::new()?;

    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });

    ui.run()
}