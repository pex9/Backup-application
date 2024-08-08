mod gui; // Declare the module

use gui::MyApp; // Import the struct
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        initial_window_size: Some([740.0, 480.0].into()),
        ..Default::default()
    };

    eframe::run_native(
        "Back-up app",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    );

    Ok(())
}
