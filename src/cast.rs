use md5::{Digest, Md5};
use rust_cast::{CastDevice, ChannelMessage};
use rust_cast::channels::heartbeat::HeartbeatResponse;
use rust_cast::channels::receiver::{Application, CastDeviceApp, ReceiverResponse};
use rust_cast::message_manager::{CastMessage, CastMessagePayload};
use serde::{Deserialize, Serialize};
use models::Message;
use crate::spotify::models;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaybackSession {
    #[serde(rename = "appAllowsGrouping")]
    pub app_allows_grouping: bool,
    #[serde(rename = "isVideoContent")]
    pub is_video_content: bool,
    #[serde(rename = "streamTransferSupported")]
    pub stream_transfer_supported: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Volume {
    pub level: f64,
    pub muted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub capabilities: i64,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    pub name: String,
    pub volume: Volume,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    pub devices: Vec<Device>,
    #[serde(rename = "isMultichannel")]
    pub is_multichannel: bool,
    #[serde(rename = "playbackSession")]
    pub playback_session: PlaybackSession,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetInfoResponse {
    #[serde(rename = "requestId")]
    pub request_id: i64,
    pub status: Status,
    #[serde(rename = "type")]
    pub r#type: String,
}

const DEFAULT_RECEIVER: &'static str = "receiver-0";
const SPOTIFY_APP_ID: &'static str = "CC32E753";

pub struct Spotify {
    device: CastDevice<'static>,
    app: Application,
}

impl Spotify {
    pub fn new(host: &str, port: u16) -> Result<Self, rust_cast::errors::Error> {
        let device = CastDevice::connect_without_host_verification(host.to_string(), port)?;
        device.connection.connect(DEFAULT_RECEIVER)?;

        let app = device.receiver.launch_app(&CastDeviceApp::Custom(SPOTIFY_APP_ID.to_string()))?;

        device.connection.connect(app.transport_id.as_str())?;

        Ok(Spotify { device, app })
    }

    pub fn login(&mut self, token: String) -> Result<(), rust_cast::errors::Error> {
        self.device.receiver.broadcast_message("urn:x-cast:com.spotify.chromecast.secure.v1",
                                          &serde_json::json!({
                                              "type": "addUser",
                                              "payload": {
                                                  "blob": token,
                                                  "tokenType": "Bearer"
                                              }
                                          })
        )?;

        match self.receive()? {
            ChannelMessage::Raw(CastMessage { payload: CastMessagePayload::String(payload), .. }) => {
                let message = serde_json::from_str::<Message>(&payload)?;
                match message.r#type.as_str() {
                    "addUserError" => {
                        Err(rust_cast::errors::Error::Internal(payload))
                    }
                    _ => {
                        println!("{}", payload);
                        Ok(())
                    }
                }
            },
            ChannelMessage::Receiver(ReceiverResponse::NotImplemented(kind, message)) => {
                match kind.as_str() {
                    "LAUNCH_STATUS" => Ok(()),
                    _ => Err(rust_cast::errors::Error::Internal(format!("Received not implemented message from Chromecast: {:?}", message)))
                }

            }
            message => {
                Err(rust_cast::errors::Error::Internal(format!("Failed to receive raw message from Chromecast: {:?}", message)))
            }
        }
    }

    pub fn device_id(&mut self) -> Result<String, rust_cast::errors::Error> {
        self.device.receiver.broadcast_message("urn:x-cast:com.spotify.chromecast.secure.v1",
                                          &serde_json::json!({
                                          "type": "getInfo",
                                      })
        )?;

        match self.receive()? {
            ChannelMessage::Raw(message) => {
                match message.payload {
                    CastMessagePayload::String(payload) => {
                        println!("{}", payload);
                        let response: GetInfoResponse = serde_json::from_str(&payload)?;
                        match response.status.devices.get(0) {
                            None =>  Err(rust_cast::errors::Error::Internal("Received no devices from Chromecast".to_string())),
                            Some(device) => {
                                let mut hasher = Md5::new();
                                hasher.update(device.name.as_bytes());
                                let result = hasher.finalize();
                                Ok(hex::encode(result))
                            }
                        }
                    }
                    CastMessagePayload::Binary(_) => Err(rust_cast::errors::Error::Internal("Received unsupported raw message from Chromecast".to_string()))
                }
            },
            message => {
                Err(rust_cast::errors::Error::Internal(format!("Failed to receive raw message from Chromecast: {:?}", message)))
            }
        }
    }

    pub fn stop(&mut self) -> Result<(), rust_cast::errors::Error> {
        self.device.receiver.stop_app(self.app.session_id.clone())
    }

    fn receive(&mut self) -> Result<ChannelMessage, rust_cast::errors::Error> {
        loop {
            match self.device.receive()? {
                ChannelMessage::Heartbeat(HeartbeatResponse::Ping) => {
                    self.device.heartbeat.pong()?;
                    continue;
                }
                ChannelMessage::Receiver(ReceiverResponse::Status(_)) => {
                    continue;
                }
                message => return Ok(message)
            }
        }
    }
}
