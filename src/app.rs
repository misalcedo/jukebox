use crate::player::Observer;

slint::slint! {
export component App inherits Window {
        preferred-width: 640px;
        preferred-height: 400px;
        icon: @image-url("assets/jukebox.png");
        title: @tr("Jukebox");
        Text {
            text: "Jukebox";
            color: green;
        }
}
}

pub fn run() -> anyhow::Result<App> {
    let app = App::new()?;
    // slint::invoke_from_event_loop(|| {
    //     match playable {
    //         Playable::Track(track) => {}
    //         Playable::Playlist(playlist) => {}
    //         Playable::Album(album) => {}
    //     }
    // })?;
    Ok(app)
}
