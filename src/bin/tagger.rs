use anyhow::anyhow;
use slint::SharedString;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    ui.on_request_read_value({
        let ui_handle = ui.as_weak();

        move || {
            if let Some(ui) = ui_handle.upgrade() {
                let value = read_value().expect("Failed to read the URI from the card.");
                ui.set_value(SharedString::from(value));
            }
        }
    });

    ui.on_request_write_value({
        let ui_handle = ui.as_weak();

        move || {
            if let Some(ui) = ui_handle.upgrade() {
                write_value(ui.get_value().as_str().to_string())
                    .expect("Failed to write the URI to the card.");
            }
        }
    });

    ui.run()
}

fn read_value() -> anyhow::Result<String> {
    let ctx = pcsc::Context::establish(pcsc::Scope::User)?;
    let reader = jukebox::choose_reader(ctx)?;

    match reader.read()? {
        None => Err(anyhow!("No card is present.")),
        Some(value) => Ok(value)
    }
}

fn write_value(value: String) -> anyhow::Result<()> {
    let ctx = pcsc::Context::establish(pcsc::Scope::User)?;
    let reader = jukebox::choose_reader(ctx)?;

    if reader.write(value)? {
        return Err(anyhow!("No card is present."));
    } else {
        Ok(())
    }
}