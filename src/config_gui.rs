use eframe::egui;
use eframe::egui::{ColorImage, TextureHandle};
use std::error::Error;
use rfd::FileDialog;
use image::imageops;
use image::imageops::FilterType;
use crate::config::BackupConfig;
use std::time::{Instant, Duration};
use crate::launcher::is_enabled;

#[derive(Debug)]
pub struct BackupConfigGUI {
    config: BackupConfig,
    save_message: Option<(String, Instant)>, // use to diplay the message when save and also the time
}

impl BackupConfigGUI {
    pub fn new() -> Self {
        let mut config = BackupConfig::new();
        config.autostart_enabled=is_enabled();
        Self { config, save_message: None }
    }

    fn set_save_message(&mut self, message: String) {
        self.save_message = Some((message, Instant::now()));
    }
}

impl eframe::App for BackupConfigGUI {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let time_to_wait:u64 = 3;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {


                // Display the success message if it is within the desired time window
                if let Some((message, timestamp)) = &self.save_message {
                    if timestamp.elapsed() < Duration::from_secs(time_to_wait) {
                        ui.heading(message);
                    }
                }



                // Load image texture
                let image_path = "assets/backup-file.png";
                let texture_handle = load_image_texture(ctx, image_path, (60, 60));
                if let Some(texture) = texture_handle {
                    ui.image(texture.id(), texture.size_vec2());
                } else {
                    ui.label("Failed to load image.");
                }
                ui.add_space(3.0);
                ui.heading("Backup application");

                ui.add_space(5.0);
                // Checkbox for enabling/disabling autostart
                ui.checkbox(&mut self.config.autostart_enabled, "Enable Autostart");

                ui.add_space(5.0);

                ui.label(format!("Selected Source folder: {}", self.config.source));
                ui.add_space(3.0);
                if ui.button("Source Folder").clicked() {
                    if let Some(folder) = FileDialog::new().pick_folder() {
                        self.config.source = folder.display().to_string();
                    }
                }
                ui.add_space(3.0);
                ui.label(format!("Selected Destination folder: {}", self.config.destination));
                ui.add_space(3.0);
                if ui.button("Destination Folder").clicked() {
                    if let Some(folder) = FileDialog::new().pick_folder() {
                        self.config.destination = folder.display().to_string();
                    }
                }
                ui.add_space(3.0);

                //backup file log name
                ui.label(format!("Insert the filename of the backup log (must be not empty): {}", self.config.log_filename));
                ui.add_space(3.0);
                ui.text_edit_singleline(&mut self.config.log_filename);


                ui.add_space(3.0);
                // Convert Vec<String> to a single string separated by ';' for displaying in TextEdit
                let extensions_str = self.config.excluded_extensions.join(";");

                ui.label("Enter file extensions to exclude from backup (separated by ';'):");
                let mut input_extensions = extensions_str.clone();
                ui.add_space(3.0);
                ui.add(egui::TextEdit::multiline(&mut input_extensions)
                    .hint_text("Enter extensions separated by ';'")
                    .desired_rows(5)
                );

                // Convert the input string back to Vec<String> when the user modifies the text area
                if input_extensions != extensions_str {
                    self.config.excluded_extensions = input_extensions.split(';')
                        .map(|s| s.trim().to_string())
                        .collect();
                }

                // Convert Vec<String> to a single string separated by ';' for displaying in TextEdit
                let directories_str = self.config.excluded_directories.join(";");

                ui.label("Enter directories to exclude from backup (separated by ';'):");
                ui.add_space(3.0);
                let mut input_directories = directories_str.clone();
                ui.add(egui::TextEdit::multiline(&mut input_directories)
                    .hint_text("Enter directories separated by ';'")
                    .desired_rows(5)
                );

                // Convert the input string back to Vec<String> when the user modifies the text area
                if input_directories != directories_str {
                    self.config.excluded_directories = input_directories.split(';')
                        .map(|s| s.trim().to_string())
                        .collect();
                }



                ui.horizontal(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);

                        // Place the buttons side by side
                        if ui.button("Save options").clicked() {
                            ui.add_space(5.0);
                            match self.config.save_info() {
                                Ok(_) => self.set_save_message("Info saved successfully".to_string()),
                                Err(e) => self.set_save_message(format!("Failed to save info: {:?}", e)),
                            };
                        }
                        ui.add_space(3.0);
                        if ui.button("Close Window").clicked() {
                            _frame.close(); // Request to close the window
                        }
                    });
                });
            });
        });
    }
}

fn load_image_texture(
    ctx: &egui::Context,
    path: &str,
    size: (u32, u32),
) -> Option<TextureHandle> {
    let image = match image::open(path) {
        Ok(image) => image.to_rgba8(),
        Err(err) => {
            eprintln!("Failed to load image: {:?}", err);
            return None;
        }
    };

    let resized_image = imageops::resize(&image, size.0, size.1, FilterType::Lanczos3);
    let color_image = ColorImage::from_rgba_unmultiplied([size.0 as usize, size.1 as usize], &resized_image);

    Some(ctx.load_texture("backup-file", color_image, Default::default()))
}

pub fn run_config_gui() -> Result<(), Box<dyn Error>> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        initial_window_size: Some([640.0, 560.0].into()),
        drag_and_drop_support: false,
        resizable: false,
        ..Default::default()
    };

    eframe::run_native(
        "Back-up app",
        options,
        Box::new(|_| Box::new(BackupConfigGUI::new())),
    );

    Ok(())
}
