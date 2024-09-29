slint::slint! {
export component MainWindow inherits Window {
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
    let main_window = MainWindow::new()?;
    Ok(main_window.run()?)
}