slint::slint! {
export component MainWindow inherits Window {
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