use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Image {
    pub url: String,
    pub height: i64,
    pub width: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Followers {
    pub href: Option<String>,
    pub total: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExternalUrls {
    pub spotify: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExplicitContent {
    pub filter_enabled: bool,
    pub filter_locked: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub is_active: bool,
    pub is_private_session: bool,
    pub is_restricted: bool,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub volume_percent: i64,
    pub supports_volume: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeviceList {
    pub devices: Vec<Device>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeviceIdList {
    pub device_ids: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Offset {
    pub position: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StartPlaybackRequest {
    pub context_uri: Option<String>,
    pub uris: Option<Vec<String>>,
    pub offset: Option<Offset>,
    pub position_ms: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Disallows {
    pub resuming: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Actions {
    pub disallows: Disallows,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Artist {
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub uri: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Restrictions {
    pub reason: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Album {
    pub album_type: String,
    pub total_tracks: i64,
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
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Item {
    pub album: Album,
    pub artists: Vec<Artist>,
    pub available_markets: Vec<String>,
    pub disc_number: i64,
    pub duration_ms: i64,
    pub explicit: bool,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub restrictions: Option<Restrictions>,
    pub name: String,
    pub popularity: i64,
    pub track_number: i64,
    #[serde(rename = "type")]
    pub r#type: String,
    pub uri: String,
    pub is_local: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Context {
    #[serde(rename = "type")]
    pub r#type: String,
    pub href: String,
    pub external_urls: ExternalUrls,
    pub uri: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Track {
    album: Album,
    artists: Vec<Artist>,
    name: String,
    uri: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Owner {
    pub external_urls: ExternalUrls,
    pub followers: Followers,
    pub href: String,
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub uri: String,
    pub display_name: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Playlist {
    name: String,
    owner: Owner,
    uri: String,
}
