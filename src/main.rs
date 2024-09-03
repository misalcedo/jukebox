use std::env;

slint::include_modules!();

mod spotify;
mod token;


fn main() {
    let client_id = env::var("CLIENT_ID").expect("Missing the CLIENT_ID environment variable.");
    let token_path = env::var("TOKEN").expect("Missing the TOKEN_PATH environment variable.");

    let oauth = token::Client::new(client_id, token_path);
    let mut client = spotify::Client::new(oauth);

    let me = client.me().expect("Failed to load user");

    println!("{me:?}");

    let devices = client.get_available_devices().expect("Failed to load devices");

    println!("{devices:?}");

    let state = client.get_playback_state().expect("Failed to get playback state");

    println!("{state:?}");

}