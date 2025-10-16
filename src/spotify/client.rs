use crate::spotify::models::{Album, DeviceList, Playlist, StartPlaybackRequest, Track};
use crate::token;

#[derive(Clone)]
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

    pub async fn get_available_devices(&mut self) -> reqwest::Result<DeviceList> {
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
    ) -> reqwest::Result<()> {
        let token = self.oauth.authorization().await.unwrap_or_default();

        self.http
            .put("https://api.spotify.com/v1/me/player/play")
            .query(&device_id.map(|id| [("device_id", id)]))
            .header("Authorization", token)
            .json(request)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn pause(&mut self, device_id: Option<String>) -> reqwest::Result<()> {
        let token = self.oauth.authorization().await.unwrap_or_default();

        self.http
            .put("https://api.spotify.com/v1/me/player/pause")
            .query(&device_id.map(|id| [("device_id", id)]))
            .header("Authorization", token)
            .header("Content-Length", 0)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn skip_to_next(&mut self, device_id: Option<String>) -> reqwest::Result<()> {
        let token = self.oauth.authorization().await.unwrap_or_default();

        self.http
            .post("https://api.spotify.com/v1/me/player/next")
            .query(&device_id.map(|id| [("device_id", id)]))
            .header("Authorization", token)
            .header("Content-Length", 0)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn get_track(&mut self, id: &str) -> reqwest::Result<Track> {
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

    pub async fn get_album(&mut self, id: &str) -> reqwest::Result<Album> {
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

    pub async fn get_playlist(&mut self, id: &str) -> reqwest::Result<Playlist> {
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