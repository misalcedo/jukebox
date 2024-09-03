use md5::{Digest, Md5};
use rust_cast::{CastDevice, ChannelMessage};
use rust_cast::channels::heartbeat::HeartbeatResponse;
use rust_cast::channels::receiver::{Application, CastDeviceApp, ReceiverResponse};
use rust_cast::message_manager::CastMessagePayload;
use serde::{Deserialize, Serialize};

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
    launched: bool,
}

impl Spotify {
    pub fn new(host: &str, port: u16) -> Result<Self, rust_cast::errors::Error> {
        let device = CastDevice::connect_without_host_verification(host.to_string(), port)?;
        device.connection.connect(DEFAULT_RECEIVER)?;

        let app = device.receiver.launch_app(&CastDeviceApp::Custom(SPOTIFY_APP_ID.to_string()))?;
        let mut launched = false;

        device.connection.connect(app.transport_id.as_str())?;
        match device.receive()? {
            ChannelMessage::Receiver(ReceiverResponse::Status(_)) => (),
            ChannelMessage::Receiver(ReceiverResponse::NotImplemented(message_type, _)) if message_type.as_str() == "LAUNCH_STATUS" => {
                launched = true;
            },
            message => {
                debug(message);
                return Err(rust_cast::errors::Error::Internal("Failed to receive status message from Chromecast".to_string()))
            }
        }

        Ok(Spotify { device, app, launched })
    }

    pub fn login(&mut self, token: String) -> Result<(), rust_cast::errors::Error> {
        if self.launched {
            return Ok(());
        }

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
            ChannelMessage::Raw(message) => {
                match message.payload {
                    CastMessagePayload::String(payload) => {
                        let response: GetInfoResponse = serde_json::from_str(&payload)?;
                        println!("{:?}", response);
                        Ok(())
                    }
                    CastMessagePayload::Binary(_) => Err(rust_cast::errors::Error::Internal("Received unsupported raw message from Chromecast".to_string()))
                }
            },
            m => {
                debug(m);
                Err(rust_cast::errors::Error::Internal("Failed to receive raw message from Chromecast".to_string()))
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
            m => {
                debug(m);
                Err(rust_cast::errors::Error::Internal("Failed to receive raw message from Chromecast".to_string()))
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
                ChannelMessage::Receiver(ReceiverResponse::Status(s)) => {
                    println!("{:?}", s);
                    continue;
                }
                message => return Ok(message)
            }
        }
    }
}

fn debug(message: ChannelMessage) {
    match message {
        ChannelMessage::Heartbeat(response) => {
            println!("[Heartbeat] {:?}", response);
        }

        ChannelMessage::Connection(response) => println!("[Connection] {:?}", response),
        ChannelMessage::Media(response) => println!("[Media] {:?}", response),
        ChannelMessage::Receiver(response) => println!("[Receiver] {:?}", response),
        ChannelMessage::Raw(response) => println!(
            "Support for the following message type is not yet supported: {:?}",
            response
        ),
    }
}
