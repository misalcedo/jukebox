use crate::spotify::models::{Device, StartPlaybackRequest};
use crate::spotify::uri_parts;
use anyhow::anyhow;

pub mod card;
pub mod spotify;
pub mod token;

pub fn choose_reader(ctx: pcsc::Context) -> anyhow::Result<card::Reader> {
    let mut readers = ctx.list_readers_owned()?;
    // Look for "ACS ACR1252 1S CL Reader PICC 0"
    let reader = readers
        .pop()
        .ok_or_else(|| anyhow!("No readers are connected."))?;

    Ok(card::Reader::new(ctx, reader))
}

pub fn choose_device(client: &mut spotify::Client, name: Option<&str>) -> anyhow::Result<Device> {
    let device = client
        .get_available_devices()?
        .devices
        .into_iter()
        .find(|device| match name {
            None => true,
            Some(name) => &device.name == name,
        })
        .ok_or_else(|| anyhow!("Found no matching device"))?;

    Ok(device)
}

pub fn start_playback(
    client: &mut spotify::Client,
    device_id: String,
    uri: String,
) -> anyhow::Result<()> {
    let mut request = StartPlaybackRequest::default();

    let (category, _) =
        uri_parts(&uri).ok_or_else(|| anyhow!("Failed to extract category from URI"))?;
    match category {
        "track" => {
            request.uris = Some(vec![uri]);
        }
        "playlist" => {
            request.context_uri = Some(uri);
        }
        "album" => {
            request.context_uri = Some(uri);
        }
        _ => {
            return Err(anyhow!("Unsupported URI category"));
        }
    }

    client.play(Some(device_id), &request)?;

    // Sometimes shuffle is unable to find a playback session.
    if let Err(err) = client.shuffle(true) {
        if err.status() == Some(reqwest::StatusCode::NOT_FOUND) {
            client.shuffle(true)?;
        }
    };

    Ok(())
}
