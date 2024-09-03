use crate::spotify::models::{DeviceAuthorizationRequest, DeviceIdList, PlaybackState, StartPlaybackRequest};
use crate::token;
use reqwest::Result;

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

    pub fn token(&mut self) -> String {
        self.oauth.token()
    }

    pub fn get_available_devices(&mut self) -> Result<models::DeviceList> {
        self.http.get("https://api.spotify.com/v1/me/player/devices")
            .header("Authorization",  self.oauth.authorization())
            .send()?
            .error_for_status()?
            .json()
    }

    pub fn transfer_playback(&mut self, devices: &DeviceIdList) -> Result<()> {
        self.http.put("https://api.spotify.com/v1/me/player")
            .header("Authorization", self.oauth.authorization())
            .json(devices)
            .send()?
            .error_for_status()?;

        Ok(())
    }

    pub fn play(&mut self, device_id: Option<String>, request: &StartPlaybackRequest) -> Result<()> {
        self.http.put("https://api.spotify.com/v1/me/player/play")
            .query(&[("device_id", device_id)])
            .header("Authorization",  self.oauth.authorization())
            .json(request)
            .send()?
            .error_for_status()?;

        Ok(())
    }

    pub fn get_playback_state(&mut self) -> Result<PlaybackState> {
        self.http.get("https://api.spotify.com/v1/me/player")
            .header("Authorization",  self.oauth.authorization())
            .send()?
            .error_for_status()?
            .json()
    }

    pub fn enable_device(&mut self, device_id: String) -> Result<()> {
        self.http.post("https://spclient.wg.spotify.com/device-auth/v1/refresh")
            .header("Authority", "spclient.wg.spotify.com")
            .header("Authorization",  self.oauth.authorization())
            .json(&DeviceAuthorizationRequest {
                client_id: self.oauth.client_id(),
                device_id,
            })
            .send()?
            .error_for_status()?;

        Ok(())
    }
}