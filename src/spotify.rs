use crate::spotify::models::{
    Album, DeviceList, PlaybackState, Playlist, StartPlaybackRequest, Track,
};
use crate::token;
use reqwest::Result;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use url::Url;

pub mod models;

const MYSTERY: &'static str = "mystery";

#[derive(Debug, Clone)]
pub struct Uri {
    pub category: String,
    pub id: String,
    pub mystery: bool,
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
                    mystery: false,
                })
            }
            Some(("https", _)) => {
                let url = Url::parse(s).map_err(|_| UriParseError)?;
                let fragment = url.fragment();
                let mut path = url.path_segments().into_iter().flatten();

                match (path.next(), path.next()) {
                    (Some(category), Some(id)) => Ok(Uri {
                        category: category.to_string(),
                        id: id.to_string(),
                        mystery: fragment == Some(MYSTERY),
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
    http: reqwest::Client,
    market: String,
}

impl Client {
    pub fn new(oauth: token::Client, market: String) -> Client {
        let http = reqwest::Client::new();
        Client {
            oauth,
            http,
            market,
        }
    }

    pub async fn get_available_devices(&mut self) -> Result<DeviceList> {
        let token = self.oauth.authorization().await.unwrap_or_default();

        self.http
            .get("https://api.spotify.com/v1/me/player/devices")
            .header("Authorization", token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn play(
        &mut self,
        device_id: Option<String>,
        request: &StartPlaybackRequest,
    ) -> Result<()> {
        let token = self.oauth.authorization().await.unwrap_or_default();

        self.http
            .put("https://api.spotify.com/v1/me/player/play")
            .query(&[("device_id", device_id)])
            .header("Authorization", token)
            .json(request)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn pause(&mut self, device_id: Option<String>) -> Result<()> {
        let token = self.oauth.authorization().await.unwrap_or_default();

        self.http
            .put("https://api.spotify.com/v1/me/player/pause")
            .query(&[("device_id", device_id)])
            .header("Authorization", token)
            .header("Content-Length", 0)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn get_playback_state(&mut self) -> Result<Option<PlaybackState>> {
        let token = self.oauth.authorization().await.unwrap_or_default();

        let response = self
            .http
            .get("https://api.spotify.com/v1/me/player")
            .query(&[("market", self.market.as_str())])
            .header("Authorization", token)
            .header("Content-Length", 0)
            .send()
            .await?
            .error_for_status()?;

        if response.content_length().unwrap_or_default() == 0 {
            return Ok(None);
        }

        response.json().await
    }

    pub async fn get_track(&mut self, id: &str) -> Result<Track> {
        let token = self.oauth.authorization().await.unwrap_or_default();

        self.http
            .get(format!("https://api.spotify.com/v1/tracks/{}", id))
            .query(&[("market", self.market.as_str())])
            .header("Authorization", token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn get_album(&mut self, id: &str) -> Result<Album> {
        let token = self.oauth.authorization().await.unwrap_or_default();

        self.http
            .get(format!("https://api.spotify.com/v1/albums/{}", id))
            .query(&[("market", self.market.as_str())])
            .header("Authorization", token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn get_playlist(&mut self, id: &str) -> Result<Playlist> {
        let token = self.oauth.authorization().await.unwrap_or_default();

        self.http
            .get(format!("https://api.spotify.com/v1/playlists/{}", id))
            .query(&[("market", self.market.as_str())])
            .header("Authorization", token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}
