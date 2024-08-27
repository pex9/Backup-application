use std::error::Error;
use egui::Context;
use eframe::NativeOptions;
use crate::utils::load_icon;

const APP_NAME: &str = "EmergencyBackup";
struct ErrorGui {
    message: String,
}

impl ErrorGui {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl eframe::App for ErrorGui {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Error");
                ui.add_space(6.0);
                ui.label(&self.message);
                ui.add_space(6.0);

                // "OK" button to close the window
                if ui.button("OK").clicked() {
                    _frame.close();
                }
            });
        });
    }
}


pub fn run_error_gui(message: String) -> Result<(), Box<dyn Error>> {
    // Load an icon for the window
    let icon = load_icon("assets/backup-file.png")?;

    // Set the window options
    let options = NativeOptions {
        initial_window_size: Some(egui::vec2(220.0, 130.0)),
        drag_and_drop_support: false,
        resizable: false,
        icon_data: Some(icon),
        always_on_top: true,
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(move |_cc| Box::new(ErrorGui::new(message))),
    );
    Ok(())
}