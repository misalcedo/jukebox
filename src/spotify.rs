use crate::spotify::models::{Album, DeviceList, Playlist, StartPlaybackRequest, Track};
use crate::token;
use reqwest::Result;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use url::Url;

pub mod models;

#[derive(Debug, Clone)]
pub struct Uri {
    pub category: String,
    pub id: String,
}

impl Display for Uri {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "spotify:{}:{}", self.category, self.id)
    }
}

impl PartialEq<str> for Uri {
    fn eq(&self, other: &str) -> bool {
        let Some(("spotify", parts)) = other.split_once(":") else {
            return false;
        };
        let Some((category, id)) = parts.split_once(":") else {
            return false;
        };

        self.category == category && self.id == id
    }
}

#[derive(Debug)]
pub struct UriParseError;

impl Display for UriParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid URI")
    }
}

impl Error for UriParseError {}

impl FromStr for Uri {
    type Err = UriParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.split_once(":") {
            Some(("spotify", parts)) => {
                let (category, id) = parts.split_once(":").ok_or(UriParseError)?;

                Ok(Uri {
                    category: category.to_string(),
                    id: id.to_string(),
                })
            }
            Some(("https", _)) => {
                let url = Url::parse(s).map_err(|_| UriParseError)?;
                let mut path = url.path_segments().into_iter().flatten();

                match (path.next(), path.next()) {
                    (Some(category), Some(id)) => Ok(Uri {
                        category: category.to_string(),
                        id: id.to_string(),
                    }),
                    _ => Err(UriParseError),
                }
            }
            _ => Err(UriParseError),
        }
    }
}

pub struct Client {
    oauth: token::Client,
    http: reqwest::blocking::Client,
    market: String,
}

impl Client {
    pub fn new(oauth: token::Client, market: String) -> Client {
        let http = reqwest::blocking::Client::new();
        Client {
            oauth,
            http,
            market,
        }
    }

    pub fn get_available_devices(&mut self) -> Result<DeviceList> {
        self.http
            .get("https://api.spotify.com/v1/me/player/devices")
            .header("Authorization", self.oauth.authorization())
            .send()?
            .error_for_status()?
            .json()
    }

    pub fn play(&mut self, device_id: String, request: &StartPlaybackRequest) -> Result<()> {
        self.http
            .put("https://api.spotify.com/v1/me/player/play")
            .query(&[("device_id", device_id)])
            .header("Authorization", self.oauth.authorization())
            .json(request)
            .send()?
            .error_for_status()?;

        Ok(())
    }

    pub fn shuffle(&mut self, state: bool) -> Result<()> {
        self.http
            .put("https://api.spotify.com/v1/me/player/shuffle")
            .query(&[("state", state)])
            .header("Authorization", self.oauth.authorization())
            .send()?
            .error_for_status()?;

        Ok(())
    }

    pub fn pause(&mut self, device_id: String) -> Result<()> {
        self.http
            .put("https://api.spotify.com/v1/me/player/pause")
            .query(&[("device_id", device_id)])
            .header("Authorization", self.oauth.authorization())
            .body("")
            .send()?
            .error_for_status()?;

        Ok(())
    }

    pub fn get_track(&mut self, id: &str) -> Result<Track> {
        self.http
            .get(format!("https://api.spotify.com/v1/tracks/{}", id))
            .query(&[("market", self.market.as_str())])
            .header("Authorization", self.oauth.authorization())
            .send()?
            .error_for_status()?
            .json()
    }

    pub fn get_album(&mut self, id: &str) -> Result<Album> {
        self.http
            .get(format!("https://api.spotify.com/v1/albums/{}", id))
            .query(&[("market", self.market.as_str())])
            .header("Authorization", self.oauth.authorization())
            .send()?
            .error_for_status()?
            .json()
    }

    pub fn get_playlist(&mut self, id: &str) -> Result<Playlist> {
        self.http
            .get(format!("https://api.spotify.com/v1/playlists/{}", id))
            .query(&[("market", self.market.as_str())])
            .header("Authorization", self.oauth.authorization())
            .send()?
            .error_for_status()?
            .json()
    }
}
