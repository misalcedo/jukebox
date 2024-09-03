use reqwest::{Error, blocking::Response};
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

    pub fn me(&mut self) -> Result<models::User, Error> {
        self.http.get("https://api.spotify.com/v1/me")
            .header("Authorization",  self.oauth.token())
            .send()
            .and_then(Response::json)
    }
}