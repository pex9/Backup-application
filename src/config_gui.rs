use crate::config::BackupConfig;
use crate::launcher::is_enabled;
use eframe::egui;
use eframe::egui::ColorImage;
use image::{AnimationDecoder, DynamicImage, GenericImageView, RgbaImage};
use rfd::FileDialog;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};
use image::codecs::gif::GifDecoder;
use crate::utils::load_icon;
use crate::utils::get_project_path;

pub struct BackupConfigGUI {
    config: BackupConfig,
    save_message: Option<(String, Instant)>,
    show_instructions: bool,
    gif_frames: Vec<(Vec<ColorImage>, String)>, // Stores GIF frames and associated text
    current_frame_indices: Vec<usize>, // Current frame indices for each GIF
    last_frame_times: Vec<Instant>, // Last frame times for each GIF
    frame_duration: Duration, // Duration to show each frame
    last_repaint_time: Instant, // Last time the UI was repainted
}

impl BackupConfigGUI {
    pub fn new() -> Self {
        let mut config = BackupConfig::new();
        config.autostart_enabled = is_enabled();

        let gif_paths_and_texts = vec![
            ("assets/rectangle-command.gif", "First, draw a rectangle starting from the top left and follow all your screen of PC. You will receive the confirmation/or error thanks to audio messages."),
            ("assets/confirm.gif", "To confirm the backup, you have to draw the gesture below or click on confirm in the GUI dialog."),
            ("assets/cancel.gif", "If you don't want to go down, you have to draw the gesture below or selecting cancel in the GUI."),
        ];

        let gif_frames = gif_paths_and_texts.into_iter()
            .map(|(path, text)| {
                let frames = load_gif_frames(path).unwrap_or_else(|_| vec![]);
                (frames, text.to_string())
            })
            .collect();

        Self {
            config,
            save_message: None,
            show_instructions: false,
            gif_frames,
            current_frame_indices: vec![0; 3], // Initialize frame indices for 3 GIFs
            last_frame_times: vec![Instant::now(); 3], // Initialize last frame times for 3 GIFs
            frame_duration: Duration::from_millis(80), // Adjust duration for your GIFs
            last_repaint_time: Instant::now(), // Initialize last repaint time
        }
    }

    fn set_save_message(&mut self, message: String) {
        self.save_message = Some((message, Instant::now()));
    }

    fn update_gif_frames(&mut self) {
        let now = Instant::now();
        for (i, (frames, _)) in self.gif_frames.iter().enumerate() {
            if now.duration_since(self.last_frame_times[i]) >= self.frame_duration {
                self.last_frame_times[i] = now;
                self.current_frame_indices[i] = (self.current_frame_indices[i] + 1) % frames.len();
            }
        }
    }
}

impl eframe::App for BackupConfigGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint every frame to ensure regular updates
        ctx.request_repaint();

        let now = Instant::now();
        // Update GIF frames periodically
        if now.duration_since(self.last_repaint_time) >= self.frame_duration {
            self.update_gif_frames();
            self.last_repaint_time = now;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    // Display the success message if it is within the desired time window
                    if let Some((message, timestamp)) = &self.save_message {
                        if timestamp.elapsed() < Duration::from_secs(3) {
                            ui.heading(message);
                        }
                    }

                    ui.horizontal(|ui| {
                        // Push the button to the far right
                        let space = egui::vec2(ui.available_width() * 0.85, 0.0);
                        ui.allocate_space(space);

                        // Instructions button on the right
                        if ui.button("Instructions").clicked() {
                            self.show_instructions = !self.show_instructions;
                        }
                    });
                    ui.heading("Backup application"); // Display the heading
                    if self.show_instructions {
                        for (i, (frames, text)) in self.gif_frames.iter().enumerate() {
                            // Center each text-GIF pair horizontally
                            ui.horizontal(|ui| {
                                ui.vertical_centered(|ui| {
                                    // Display text above GIF
                                    ui.label(text);
                                    if let Some(frame) = frames.get(self.current_frame_indices[i]) {
                                        let size = egui::vec2(frame.width() as f32, frame.height() as f32);
                                        let texture = ctx.load_texture(
                                            format!("instructions-gif-{}", i), // Unique texture name for each GIF
                                            frame.clone(),
                                            Default::default(),
                                        );
                                        ui.image(texture.id(), size);
                                    } else {
                                        ui.label("Failed to load GIF frame.");
                                    }
                                    ui.add_space(10.0); // Add space after each GIF
                                });
                            });
                        }
                    }

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

                    ui.label(format!("Insert the filename of the backup log (must be not empty): {}", self.config.log_filename));
                    ui.add_space(3.0);
                    ui.text_edit_singleline(&mut self.config.log_filename);

                    ui.add_space(3.0);
                    let extensions_str = self.config.excluded_extensions.join("\n");

                    ui.label("Enter file extensions to exclude from backup (on different lines):");
                    let mut input_extensions = extensions_str.clone();
                    ui.add_space(3.0);
                    ui.add(egui::TextEdit::multiline(&mut input_extensions).hint_text("Enter extensions on different lines").desired_rows(5));

                    // Convert the input string back to Vec<String> when the user modifies the text area
                    if input_extensions != extensions_str {
                        self.config.excluded_extensions = input_extensions.split('\n').map(|s| s.trim().to_string()).collect();
                    }

                    // Convert Vec<String> to a single string separated by ';' for displaying in TextEdit
                    let directories_str = self.config.excluded_directories.join("\n");

                    ui.label("Enter directories to exclude from backup (on different lines):");
                    ui.add_space(3.0);
                    let mut input_directories = directories_str.clone();
                    ui.add(egui::TextEdit::multiline(&mut input_directories).hint_text("Enter directories on different lines").desired_rows(5));

                    // Convert the input string back to Vec<String> when the user modifies the text area
                    if input_directories != directories_str {
                        self.config.excluded_directories = input_directories.split('\n').map(|s| s.trim().to_string()).collect();
                    }
                    ui.add_space(10.0);
                    // Place the buttons side by side
                    ui.horizontal(|ui| {
                        // Push the button to the far right
                        let space = egui::vec2(ui.available_width() * 0.35, 0.0);
                        ui.allocate_space(space);
                        if ui.button("Save options").clicked() {
                            match self.config.save_info() {
                                Ok(_) => {
                                    self.set_save_message("Info saved successfully".to_string())
                                }
                                Err(e) => {
                                    self.set_save_message(format!("Failed to save info: {}", e))
                                }
                            };
                        }
                        let space = egui::vec2(ui.available_width() * 0.05, 0.0);
                        ui.allocate_space(space);
                        if ui.button("Close Window").clicked() {
                            _frame.close();
                        }
                    });
                });
            });
        });
    }
}

fn load_gif_frames(path: &str) -> Result<Vec<ColorImage>, Box<dyn Error>> {
    let file = File::open(get_project_path(path))?;
    let reader = BufReader::new(file);
    let decoder = GifDecoder::new(reader)?;
    let mut frames = decoder.into_frames();
    let mut color_images = Vec::new();

    while let Some(Ok(frame)) = frames.next() {
        let image = DynamicImage::from(frame.into_buffer());
        let (width, height) = image.dimensions();
        let rgba_image: RgbaImage = image.to_rgba8();
        let color_image = ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            &rgba_image.into_raw(),
        );
        color_images.push(color_image);
    }

    Ok(color_images)
}

pub fn run_config_gui() -> Result<(), Box<dyn Error>> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let icon = load_icon("assets/backup-file.png")?;
    let options = eframe::NativeOptions {
        initial_window_size: Some([640.0, 560.0].into()),
        drag_and_drop_support: false,
        resizable: false,
        icon_data: Some(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Back-up app",
        options,
        Box::new(|_| Box::new(BackupConfigGUI::new())),
    );

    Ok(())
}
