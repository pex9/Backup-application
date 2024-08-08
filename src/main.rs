use eframe::egui;
use eframe::egui::{ColorImage, TextureHandle};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use image::imageops;
use image::imageops::FilterType;
use rfd::FileDialog;

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

struct MyApp {
    source: String,
    destination: String,
    texture: Option<TextureHandle>,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let texture = Self::load_image_texture(cc, "assets/backup-file.png", (100, 100));
        Self {
            source: "".to_string(),
            destination: "".to_string(),
            texture,
        }
    }

    fn load_image_texture(
        cc: &eframe::CreationContext<'_>,
        path: &str,
        size: (u32, u32), // Target width and height
    ) -> Option<TextureHandle> {
        // Load the image from the specified path
        let mut image = match image::open(path) {
            Ok(image) => image.to_rgba8(),
            Err(err) => {
                eprintln!("Failed to load image: {:?}", err);
                return None;
            }
        };

        // Resize the image
        image = imageops::resize(&image, size.0, size.1, FilterType::Lanczos3);

        // Convert the image into a ColorImage for egui
        let color_image = ColorImage::from_rgba_unmultiplied([size.0 as usize, size.1 as usize], &image);

        // Create a texture handle for the image
        Some(cc.egui_ctx.load_texture("backup-file", color_image, Default::default()))
    }
    //function to save information of the source/destination directory
    fn save_info(&self) -> Result<bool, Box<dyn Error>> {
        // Define the path where you want to save the information

        let path = "config/backup_info.txt";
        let mut file = File::create(path)?;

        // Write the source and destination paths to the file
        writeln!(file, "Source: {}", self.source)?;
        writeln!(file, "Destination: {}", self.destination)?;
        println!("si ho salvato il path è {}", path);
        Ok(true)
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Display the image if it's loaded
            ui.horizontal(|ui| {
                if let Some(texture) = &self.texture {
                    ui.image(texture, texture.size_vec2()); // Display the image
                } else {
                    ui.label("Failed to load image.");
                }
                ui.heading("Back Up Application"); // Text next to the image
            });

            ui.label("Select a source folder for backup:");
            if ui.button("Source Folder").clicked() {
                if let Some(folder) = FileDialog::new().pick_folder() {
                    self.source = folder.display().to_string();
                }
            }
            ui.label(format!("Selected Source folder: {}", self.source));

            ui.label("Select a destination folder for backup:");
            if ui.button("Destination Folder").clicked() {
                if let Some(folder) = FileDialog::new().pick_folder() {
                    self.destination = folder.display().to_string();
                }
            }
            ui.label(format!("Selected Destination folder: {}", self.destination));
            if ui.button("Save Info").clicked() {
                match self.save_info(){
                    Ok(_) => ui.label("Info saved successfully."),
                    Err(e) => ui.label(format!("Failed to save info: {:?}", e)),
                };
            }



        });
    }
}
