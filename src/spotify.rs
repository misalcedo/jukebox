use std::io::Read;
use reqwest::{Result, blocking::Response};
use crate::spotify::models::{DeviceIdList, PlaybackState, StartPlaybackRequest};
use crate::token;

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

    pub fn me(&mut self) -> Result<models::User> {
        self.http.get("https://api.spotify.com/v1/me")
            .header("Authorization",  self.oauth.token())
            .send()
            .and_then(Response::json)
    }

    pub fn get_available_devices(&mut self) -> Result<models::DeviceList> {
        self.http.get("https://api.spotify.com/v1/me/player/devices")
            .header("Authorization",  self.oauth.token())
            .send()
            .and_then(Response::json)
    }

    pub fn transfer_playback(&mut self, devices: &DeviceIdList) -> Result<()> {
        self.http.put("https://api.spotify.com/v1/me/player")
            .header("Authorization",  self.oauth.token())
            .json(devices)
            .send()
            .and_then(Response::json)
    }

    pub fn play(&mut self, request: &StartPlaybackRequest) -> Result<()> {
        self.http.put("https://api.spotify.com/v1/me/player/play")
            .header("Authorization",  self.oauth.token())
            .json(request)
            .send()
            .and_then(Response::json)

    }

    pub fn get_playback_state(&mut self) -> Result<models::PlaybackState> {
        self.http.get("https://api.spotify.com/v1/me/player")
            .header("Authorization",  self.oauth.token())
            .send()
            .and_then(Response::json)

    }
}