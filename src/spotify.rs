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
            return false
        };
        let Some((category, id)) = parts.split_once(":") else {
            return false
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
        Self::try_from(s).map_err(|_| UriParseError)
    }
}

impl<'a> TryFrom<&'a str> for Uri {
    type Error = &'a str;

    fn try_from(value: &'a str) -> std::result::Result<Self, Self::Error> {
        let Some(("spotify", parts)) = value.split_once(":") else {
            return Err(value);
        };
        let (category, id) = parts.split_once(":").ok_or(value)?;

        Ok(Uri {
            category: category.to_string(),
            id: id.to_string(),
        })
    }
}

impl TryFrom<Url> for Uri {
    type Error = Url;

    fn try_from(value: Url) -> std::result::Result<Self, Self::Error> {
        let mut path = value.path_segments().into_iter().flatten();
        match (value.scheme(), path.next(), path.next()) {
            ("spotify", None, None) => match Self::try_from(value.as_str()) {
                Ok(uri) => Ok(uri),
                Err(_) => Err(value),
            },
            ("https", Some(category), Some(id)) => Ok(Uri {
                category: category.to_string(),
                id: id.to_string(),
            }),
            _ => Err(value),
        }
    }
}

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

    pub fn shuffle(&mut self, state: bool) -> Result<()> {
        self.http
            .put("https://api.spotify.com/v1/me/player/shuffle")
            .query(&[("state", state)])
            .header("Authorization", self.oauth.authorization())
            .body("")
            .send()?
            .error_for_status()?;

        Ok(())
    }

    pub fn get_track(&mut self, id: &str) -> Result<Track> {
        self.http
            .put(format!("https://api.spotify.com/v1/tracks/{}", id))
            .header("Authorization", self.oauth.authorization())
            .body("")
            .send()?
            .error_for_status()?
            .json()
    }

    pub fn get_album(&mut self, id: &str) -> Result<Album> {
        self.http
            .put(format!("https://api.spotify.com/v1/albums/{}", id))
            .header("Authorization", self.oauth.authorization())
            .body("")
            .send()?
            .error_for_status()?
            .json()
    }

    pub fn get_playlist(&mut self, id: &str) -> Result<Playlist> {
        self.http
            .put(format!("https://api.spotify.com/v1/playlists/{}", id))
            .header("Authorization", self.oauth.authorization())
            .body("")
            .send()?
            .error_for_status()?
            .json()
    }
}
