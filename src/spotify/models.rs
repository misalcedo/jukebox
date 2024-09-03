use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub url: String,
    pub height: i64,
    pub width: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Followers {
    pub href: Option<String>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalUrls {
    pub spotify: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExplicitContent {
    pub filter_enabled: bool,
    pub filter_locked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
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