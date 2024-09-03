use std::env;

mod spotify;
mod token;

fn main() {
    let client_id = env::var("CLIENT_ID").expect("Missing the CLIENT_ID environment variable.");
    let token_path = env::var("TOKEN").expect("Missing the TOKEN_PATH environment variable.");

    let oauth = token::Client::new(client_id, token_path);
    let mut client = spotify::Client::new(oauth);
    let mut spotify = cast::Spotify::new("192.168.1.22", 8009).expect("Failed to connect to Chromecast");

    let device_id = spotify.device_id().expect("Failed to get device id");
    println!("Device id: {}", device_id);

    spotify.login(client.token()).expect("Failed to login to Spotify");

    // client.enable_device("a16207e6e05f6f9ac1cee93e0e3ad3c0".to_string());//.expect("Failed to enable device");
    // client.transfer_playback(&DeviceIdList {
    //     device_ids: vec![device_id]
    // }).expect("Failed to transfer playback");
    //
    spotify.stop().expect("Failed to stop Spotify");

    let devices = client.get_available_devices().expect("Failed to load devices");

    println!("{devices:?}");

    // let state = client.get_playback_state().expect("Failed to get playback state");

    // println!("{state:?}");
    // Some("a16207e6e05f6f9ac1cee93e0e3ad3c0".to_string())
    // client.play(Some("a16207e6e05f6f9ac1cee93e0e3ad3c0".to_string()), &StartPlaybackRequest {
    //     context_uri: None,
    //     offset: None,
    //     uris: vec!["spotify:track:6b2HYgqcK9mvktt4GxAu72".to_string()],
    //     position_ms: 0,
    // }).expect("Failed to play");
}
