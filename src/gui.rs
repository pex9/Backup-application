use eframe::egui;
use eframe::egui::{ColorImage, TextureHandle};
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use image::imageops;
use image::imageops::FilterType;
use rfd::FileDialog;
use sysinfo::{Pid, System};
use crate::utils::start_monitor;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FileExtension {
    Txt,
    Pdf,
    Png,
    Jpg,
    Mp4,
    All,
}

impl FileExtension {
    fn all() -> &'static [FileExtension] {
        &[
            FileExtension::All,
            FileExtension::Txt,
            FileExtension::Pdf,
            FileExtension::Png,
            FileExtension::Jpg,
            FileExtension::Mp4,
        ]
    }

    fn to_string(&self) -> &'static str {
        match self {
            FileExtension::Txt => "txt",
            FileExtension::Pdf => "pdf",
            FileExtension::Png => "png",
            FileExtension::Jpg => "jpg",
            FileExtension::Mp4 => "mp4",
            FileExtension::All => "All",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "All" => Some(FileExtension::All),
            "txt" => Some(FileExtension::Txt),
            "pdf" => Some(FileExtension::Pdf),
            "png" => Some(FileExtension::Png),
            "jpg" => Some(FileExtension::Jpg),
            "mp4" => Some(FileExtension::Mp4),
            _ => None,
        }
    }
}


pub struct MyApp {
    source: String,
    destination: String,
    texture: Option<TextureHandle>,
    selected_extensions: Vec<FileExtension>,
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {

        start_monitor();
        let mut app = Self {
            source: "".to_string(),
            destination: "".to_string(),
            texture: Self::load_image_texture(cc, "assets/backup-file.png", (100, 100)),
            selected_extensions: Vec::new(),
        };

        // Load previously saved information
        app.load_info();
        app
    }

    fn load_image_texture(
        cc: &eframe::CreationContext<'_>,
        path: &str,
        size: (u32, u32),
    ) -> Option<TextureHandle> {
        let mut image = match image::open(path) {
            Ok(image) => image.to_rgba8(),
            Err(err) => {
                eprintln!("Failed to load image: {:?}", err);
                return None;
            }
        };

        image = imageops::resize(&image, size.0, size.1, FilterType::Lanczos3);
        let color_image = ColorImage::from_rgba_unmultiplied([size.0 as usize, size.1 as usize], &image);

        Some(cc.egui_ctx.load_texture("backup-file", color_image, Default::default()))
    }

    fn save_info(&self) -> Result<(), Box<dyn Error>> {
        let path = "config/backup_info.txt";
        let mut file = File::create(path)?;

        writeln!(file, "Source: {}", self.source)?;
        writeln!(file, "Destination: {}", self.destination)?;

        // Write selected extensions
        for ext in &self.selected_extensions {
            writeln!(file, "{}", ext.to_string())?;
        }

        Ok(())
    }

    fn load_info(&mut self) {
        let path = "config/backup_info.txt";
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            for (i, line) in reader.lines().enumerate() {
                if let Ok(line) = line {
                    if i == 0 {
                        if line.starts_with("Source: ") {
                            self.source = line["Source: ".len()..].to_string();
                        }
                    } else if i == 1 {
                        if line.starts_with("Destination: ") {
                            self.destination = line["Destination: ".len()..].to_string();
                        }
                    } else {
                        if let Some(ext) = FileExtension::from_str(&line) {
                            self.selected_extensions.push(ext);
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Backup application");

                if let Some(texture) = &self.texture {
                    ui.image(texture, texture.size_vec2());
                } else {
                    ui.label("Failed to load image.");
                }

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

                ui.label("Select file extensions to backup:");

                for extension in FileExtension::all() {
                    let mut selected = self.selected_extensions.contains(extension);
                    if ui.checkbox(&mut selected, extension.to_string()).clicked() {
                        if selected {
                            if extension.to_string() == "All".to_string() {
                                for extension in FileExtension::all() {
                                    if !self.selected_extensions.contains(&*extension)
                                    {
                                        self.selected_extensions.push(*extension);
                                    }
                                }
                            } else {
                                self.selected_extensions.push(*extension);
                            }
                        } else {
                            if extension.to_string() == "All".to_string() {
                                for extension in FileExtension::all() {
                                    self.selected_extensions.retain(|&ext| ext != *extension);
                                }
                            } else {
                                self.selected_extensions.retain(|&ext| ext != *extension);
                            }
                        }
                    }
                }


                if ui.button("Save options").clicked() {
                    match self.save_info() {
                        Ok(_) => ui.label("Info saved successfully."),
                        Err(e) => ui.label(format!("Failed to save info: {:?}", e)),
                    };
                }
                if ui.button("Close Window").clicked() {
                    _frame.close(); // Request to close the window
                }
            });
        });
    }
}
