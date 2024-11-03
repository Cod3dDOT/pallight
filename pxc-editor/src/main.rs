mod app;
mod chunk;
mod image;

use app::PixelEditor;

fn main() -> Result<(), eframe::Error> {
    lib_pxc::init_logging();

    let app = PixelEditor::new();
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "PXC Editor",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )?;

    Ok(())
}
