use crate::player::{Observer, Playable};
use slint::{SharedString, Weak};

slint::slint! {
export component MainWindow inherits Window {
        in property <string> state;

        preferred-width: 640px;
        preferred-height: 400px;

        icon: @image-url("assets/jukebox.png");
        title: @tr("Jukebox");

        Text {
            text: state;
            color: green;
        }
}
}

pub struct Window {
    main_window: MainWindow,
}

impl Window {
    pub fn new() -> anyhow::Result<Self> {
        let main_window = MainWindow::new()?;
        main_window.set_state(SharedString::from("Waiting to play"));
        Ok(Self { main_window })
    }

    pub fn observer(&self) -> impl Observer {
        self.main_window.as_weak()
    }

    pub fn run(&self) -> anyhow::Result<()> {
        self.main_window.run()?;
        Ok(())
    }
}

impl Observer for Weak<MainWindow> {
    fn on_playback_started(&self, playable: Playable) {
        if let Err(e) = self.upgrade_in_event_loop(|app|
            match playable {
                Playable::Track(track) => {
                    app.set_state(slint::format!("Playing {}", track.name));
                }
                Playable::Playlist(playlist) => {
                    app.set_state(slint::format!("Playing {}", playlist.name));
                }
                Playable::Album(album) => {
                    app.set_state(slint::format!("Playing {}", album.name));
                }
            }) {
            tracing::error!(%e, "Failed to update the UI");
        }
    }

    fn on_playback_paused(&self) {
        if let Err(e) = self.upgrade_in_event_loop(|app| {
            app.set_state(SharedString::from("Paused"));
        }) {
            tracing::error!(%e, "Failed to update the UI");
        }
    }
}
