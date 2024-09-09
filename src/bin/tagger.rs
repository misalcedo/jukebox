use slint::SharedString;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    ui.on_request_read_value({
        let ui_handle = ui.as_weak();
        let read_ctx =
            pcsc::Context::establish(pcsc::Scope::User).expect("Failed to establish context");
        let reader = jukebox::choose_reader(read_ctx).expect("Failed to choose a card reader.");

        move || {
            if let Some(ui) = ui_handle.upgrade() {
                if let Some(value) = reader
                    .read()
                    .expect("Failed to read the URI from the card.")
                {
                    ui.set_value(SharedString::from(value));
                }
            }
        }
    });

    ui.on_request_write_value({
        let ui_handle = ui.as_weak();
        let write_ctx =
            pcsc::Context::establish(pcsc::Scope::User).expect("Failed to establish context");
        let reader = jukebox::choose_reader(write_ctx).expect("Failed to choose a card reader.");

        move || {
            if let Some(ui) = ui_handle.upgrade() {
                reader
                    .write(ui.get_value().as_str().to_string())
                    .expect("Failed to write the URI to the card.");
            }
        }
    });

    ui.run()
}
