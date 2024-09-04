use crate::spotify::models::{DeviceList, StartPlaybackRequest};
use crate::token;
use reqwest::Result;
use url::Url;

pub mod models;

pub struct Client {
    oauth: token::Client,
    http: reqwest::blocking::Client,
}

impl Client {
    pub fn new(oauth: token::Client) -> Client {
        let http = reqwest::blocking::Client::new();
        Client { oauth, http }
    }

    pub fn get_available_devices(&mut self) -> Result<DeviceList> {
        self.http
            .get("https://api.spotify.com/v1/me/player/devices")
            .header("Authorization", self.oauth.authorization())
            .send()?
            .error_for_status()?
            .json()
    }

    pub fn play(
        &mut self,
        device_id: Option<String>,
        request: &StartPlaybackRequest,
    ) -> Result<()> {
        self.http
            .put("https://api.spotify.com/v1/me/player/play")
            .query(&[("device_id", device_id)])
            .header("Authorization", self.oauth.authorization())
            .json(request)
            .send()?
            .error_for_status()?;

        Ok(())
    }
}

pub fn normalize_track(uri: &Url) -> Option<String> {
    match uri.scheme() {
        "spotify" => Some(uri.to_string()),
        "https" => {
            let track = uri
                .path_segments()
                .into_iter()
                .flatten()
                .last()
                .unwrap_or_default();
            Some(format!("spotify:track:{}", track))
        }
        _ => None,
    }
}
