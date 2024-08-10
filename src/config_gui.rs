use eframe::egui;
use eframe::egui::{ColorImage, TextureHandle};
use serde::{Serialize, Deserialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader};
use std::path::Path;
use rfd::FileDialog;
use image::{imageops};
use image::imageops::FilterType;
use serde_json;

#[derive(Serialize, Deserialize, Default)]
pub struct MyApp {
    pub source: String,
    pub destination: String,
    pub selected_extensions: Vec<String>,
    pub excluded_directories: Vec<String>,
}

impl MyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            source: "".to_string(),
            destination: "".to_string(),
            selected_extensions: Vec::new(),
            excluded_directories: Vec::new(),
        };

        // Load previously saved information
        app.load_info();
        app
    }

    fn save_info(&self) -> Result<(), Box<dyn Error>> {
        let path = "config/backup_info.json";
        let file = File::create(path)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }

    fn load_info(&mut self) {
        let path = "config/backup_info.json";
        if Path::new(path).exists() {
            let file = File::open(path).expect("Unable to open file");
            let reader = BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(loaded_info) => *self = loaded_info,
                Err(e) => {
                    eprintln!("Error loading JSON: {:?}", e);
                    // Handle JSON parsing errors by initializing with default values
                    self.selected_extensions = Vec::new();
                    self.excluded_directories = Vec::new();
                }
            }
        } else {
            // File does not exist; initialize with default values
            self.selected_extensions = Vec::new();
            self.excluded_directories = Vec::new();
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
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

                ui.label(format!("Selected Source folder: {}", self.source));
                ui.add_space(3.0);
                if ui.button("Source Folder").clicked() {
                    if let Some(folder) = FileDialog::new().pick_folder() {
                        self.source = folder.display().to_string();
                    }
                }
                ui.add_space(3.0);
                ui.label(format!("Selected Destination folder: {}", self.destination));
                ui.add_space(3.0);
                if ui.button("Destination Folder").clicked() {
                    if let Some(folder) = FileDialog::new().pick_folder() {
                        self.destination = folder.display().to_string();
                    }
                }
                ui.add_space(3.0);
                // Convert Vec<String> to a single string separated by ';' for displaying in TextEdit
                let extensions_str = self.selected_extensions.join(";");

                ui.label("Enter file extensions to backup (separated by ';'):");
                let mut input_extensions = extensions_str.clone();
                ui.add_space(3.0);
                ui.add(egui::TextEdit::multiline(&mut input_extensions)
                    .hint_text("Enter extensions separated by ';'")
                    .desired_rows(5)
                );

                // Convert the input string back to Vec<String> when the user modifies the text area
                if input_extensions != extensions_str {
                    self.selected_extensions = input_extensions.split(';')
                        .map(|s| s.trim().to_string())
                        .collect();
                }

                // Convert Vec<String> to a single string separated by ';' for displaying in TextEdit
                let directories_str = self.excluded_directories.join(";");

                ui.label("Enter directories to exclude from backup (separated by ';'):");
                ui.add_space(3.0);
                let mut input_directories = directories_str.clone();
                ui.add(egui::TextEdit::multiline(&mut input_directories)
                    .hint_text("Enter directories separated by ';'")
                    .desired_rows(5)
                );

                // Convert the input string back to Vec<String> when the user modifies the text area
                if input_directories != directories_str {
                    self.excluded_directories = input_directories.split(';')
                        .map(|s| s.trim().to_string())
                        .collect();
                }
                ui.horizontal(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(10.0);

                            // Place the buttons side by side
                            if ui.button("Save options").clicked() {
                                ui.add_space(5.0);
                                match self.save_info() {
                                    Ok(_) => ui.label("Info saved successfully."),
                                    Err(e) => ui.label(format!("Failed to save info: {:?}", e)),
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

    // Aggiungi l'applicazione all'avvio di Windows release
    // add_to_startup()?;


    let options = eframe::NativeOptions {
        initial_window_size: Some([640.0, 480.0].into()),
        drag_and_drop_support: false,
        resizable: false,
        ..Default::default()
    };

    eframe::run_native(
        "Back-up app",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    );

    Ok(())
}