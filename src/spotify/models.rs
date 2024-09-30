use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Image {
    pub url: String,
    pub height: Option<u64>,
    pub width: Option<u64>,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Followers {
    pub href: Option<String>,
    pub total: u64,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct ExternalUrls {
    pub spotify: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct ExplicitContent {
    pub filter_enabled: bool,
    pub filter_locked: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct User {
    pub country: String,
    pub display_name: String,
    pub email: String,
    pub explicit_content: ExplicitContent,
    pub external_urls: ExternalUrls,
    pub followers: Followers,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub product: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub uri: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Device {
    pub id: String,
    pub is_active: bool,
    pub is_private_session: bool,
    pub is_restricted: bool,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub volume_percent: u64,
    pub supports_volume: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct DeviceList {
    pub devices: Vec<Device>,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct DeviceIdList {
    pub device_ids: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Offset {
    pub position: u64,
}

impl Offset {
    pub fn random(value: u64) -> Self {
        let mut rng = rand::thread_rng();
        let position = rng.gen_range(0..value);

        Self {
            position
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct StartPlaybackRequest {
    pub context_uri: Option<String>,
    pub uris: Option<Vec<String>>,
    pub offset: Option<Offset>,
    pub position_ms: u64,
}

impl From<Vec<String>> for StartPlaybackRequest {
    fn from(value: Vec<String>) -> Self {
        Self {
            context_uri: None,
            uris: Some(value),
            offset: None,
            position_ms: 0,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Disallows {
    pub resuming: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Actions {
    pub disallows: Disallows,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Artist {
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub uri: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Restrictions {
    pub reason: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Album {
    pub album_type: String,
    pub total_tracks: u64,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub name: String,
    pub release_date: String,
    pub release_date_precision: String,
    pub restrictions: Option<Restrictions>,
    #[serde(rename = "type")]
    pub r#type: String,
    pub uri: String,
    pub artists: Vec<Artist>,
    pub tracks: Option<AlbumTracks>,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct AlbumTracks {
    pub limit: u64,
    pub total: u64,
    pub items: Vec<AlbumTrackItem>,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct AlbumTrackItem {
    pub is_local: bool,
    pub artists: Vec<Artist>,
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Item {
    pub album: Album,
    pub artists: Vec<Artist>,
    pub disc_number: u64,
    pub duration_ms: u64,
    pub explicit: bool,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub restrictions: Option<Restrictions>,
    pub name: String,
    pub popularity: u64,
    pub track_number: u64,
    #[serde(rename = "type")]
    pub r#type: String,
    pub uri: String,
    pub is_local: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Context {
    #[serde(rename = "type")]
    pub r#type: String,
    pub href: String,
    pub external_urls: ExternalUrls,
    pub uri: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Track {
    pub album: Album,
    pub artists: Vec<Artist>,
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Owner {
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub uri: String,
    pub display_name: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct PlaylistTracks {
    pub limit: u64,
    pub total: u64,
    pub items: Vec<PlaylistTrackItem>,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct PlaylistTrackItem {
    pub is_local: bool,
    pub track: Track,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Playlist {
    pub name: String,
    pub owner: Owner,
    pub uri: String,
    pub images: Vec<Image>,
    pub tracks: PlaylistTracks,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct PlaybackState {
    pub device: Device,
    pub repeat_state: String,
    pub shuffle_state: bool,
    pub smart_shuffle: bool,
    pub context: Context,
    pub timestamp: u64,
    pub progress_ms: u64,
    pub is_playing: bool,
    pub item: Item,
    pub currently_playing_type: String,
    pub actions: Actions,
}