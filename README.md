# JukeBox

A jukebox application for macOS and Windows that uses NFC tags to play music from Spotify.
The tags are encoded with the Spotify URI of the song, album or playlist that should be played.

## Requirements

1. You will need a premium Spotify account.
2. You will need an NFC reader that is supported by the application.
3. You will need NFC tags that are supported by the application.
4. You will need a macOS or Windows computer.
5. You will need a [Spotify developer application](https://developer.spotify.com/dashboard).

## Usage

```console
git clone git@github.com:misalcedo/jukebox.git
cd jukebox
cargo install --path .

export JUKEBOX_TOKEN_CACHE="$HOME/.spotify"
export JUKEBOX_CLIENT_ID="YOUR_CLIENT_ID"
export JUKEBOX_MARKET="US"
export JUKEBOX_DEVICE="$(scutil --get ComputerName)"
export JUKEBOX_ADDRESS="0.0.0.0:5853"

jukebox
```

## Login

When you run the application for the first time, go to the address you configured in the `$JUKEBOX_ADDRESS` environment
variable.
Then, click on the login link and follow the instructions.
The application will store the token in the file you configured in the `$JUKEBOX_TOKEN` environment variable.
In order for the login to work, `http://$JUKEBOX_ADDRESS/callback` needs to be registered as an endpoint on the Spotify
developer application.

## Supported Devices

### Readers

- ACR1252U

### NFC Tags

- NXP NTAG216

## Icon

The jukebox icon is made by [Freepik](https://www.flaticon.com/authors/freepik)
from [www.flaticon.com](https://www.flaticon.com/)

## Resources

- https://developer.spotify.com/documentation/web-api/tutorials/code-pkce-flow
- https://developer.spotify.com/documentation/web-api/concepts/scopes
- https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFBundles/BundleTypes/BundleTypes.html
- https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFBundles/BundleTypes/BundleTypes.html#//apple_ref/doc/uid/10000123i-CH101-SW13
- https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPRuntimeConfig/Articles/EnvironmentVars.html#//apple_ref/doc/uid/20002093-BCIJIJBH
- https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/LaunchServicesKeys.html#//apple_ref/doc/uid/TP40009250-SW1
- https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/GeneralPurposeKeys.html#//apple_ref/doc/uid/TP40009253-SW1
