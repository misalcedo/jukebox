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

pub fn run() -> anyhow::Result<()> {
    let app = App::new()?;
    Ok(app.run()?)
}