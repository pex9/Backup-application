mod utils;

mod backup;
mod config;
mod config_gui;
mod confirm_gui;
mod launcher;


use std::sync::{Arc, Mutex};
use std::{env, thread};

use config::CONFIG_FILE_PATH;
use config_gui::run_config_gui;
use confirm_gui::{run_confirm_gui, Choice};
use mouse::Mouse;
use utils::{abort_backup, get_screensize, perform_backup};
use winit::event_loop;

mod mouse;
mod sys;
pub mod types;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 && args[1] == "--config" {
        main_configuration();
    } else if args.len() == 2 && args[1] == "--screensize" {
        let (width, height) = main_get_screensize();
        println!("{}-{}", width, height);
    } else {
        main_background();
    }
}

fn main_background() {
    if !std::path::Path::new(CONFIG_FILE_PATH).exists() {
        println!("First launch of the application: no configuration found. Please run the program with the --config flag to configure it.");
        return;
    }
    utils::start_monitor();
    let mut mouse = Mouse::new();
    let screensize = get_screensize();
    loop {
        let pos = mouse.get_position().unwrap();
        if pos.x == 0 && pos.y == 0 {
            if mouse.rectangle_write((screensize.0 as i32) - 1, (screensize.1 as i32) - 1).unwrap() {
                gesture_identified();
            }
            thread::sleep(std::time::Duration::from_millis(100));
        } else {
            thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}

fn gesture_identified() {
    let mut mouse = Mouse::new();
    let controller = Arc::new(Mutex::new(false));
    let cont_gesture = Arc::clone(&controller);
    thread::spawn(move || {
        let controller = Arc::clone(&cont_gesture);
        thread::sleep(std::time::Duration::from_secs(1));
        if mouse.confirm(cont_gesture).unwrap() {
            println!("Backup started from gesture");
            perform_backup(controller).expect("Failed to perform backup");
        } else {
            println!("Backup aborted from gesture");
            abort_backup(controller);
        }
    });

    // Start GUI confirmation
    gui_confirmation(controller);
}

fn gui_confirmation(controller: Arc<Mutex<bool>>) {
    let (sender, receiver) = std::sync::mpsc::channel();
    let controller2 = Arc::clone(&controller);

    thread::spawn(move || {
        match receiver.recv() {
            Ok(choice) => {
                match choice {
                    Choice::Yes => {
                        println!("Backup started from GUI");
                        perform_backup(controller).expect("Failed to perform backup");
                    }
                    Choice::No => {
                        println!("Backup aborted from GUI");
                        abort_backup(controller);
                    }
                    Choice::CloseGui => {
                        println!("Close Gui Backup");
                        // no error code provided
                    }
                }
                #[cfg(target_os = "macos")]
                {
                    std::process::exit(0);
                }
            }
            Err(e) => {
                println!("Backup aborted 2: {:?}", e);
                abort_backup(controller);
                std::process::exit(0);
            }
        }
    });

    run_confirm_gui(sender, controller2);
}

fn main_configuration() {
    let conf_path = std::path::Path::new(CONFIG_FILE_PATH);
    if conf_path.parent().is_none() || !conf_path.parent().unwrap().exists() {
        if let Err(err) = std::fs::create_dir_all(conf_path.parent().unwrap()) {
            eprintln!("Failed to create config folder: {}", err);
        }
        println!("New configuration created");
    }
    run_config_gui().unwrap();
}

fn main_get_screensize() -> (u32, u32) {
    let event_loop = event_loop::EventLoop::new();
    let primary_monitor = event_loop.primary_monitor().unwrap();
    let physical_size = primary_monitor.size();
    let scale_factor = primary_monitor.scale_factor();
    let monitor_size = physical_size.to_logical(scale_factor);
    let width = monitor_size.width;
    let height = monitor_size.height;
    (width, height)
}
/*
extern crate image;
extern crate gif;

use image::{Rgba, RgbaImage, ImageBuffer};
use gif::{Encoder, Frame, Repeat};
use std::fs::File;
use std::error::Error;

fn draw_character(img: &mut RgbaImage, x: u32, y: u32, char: char) {
    let char_color = Rgba([0, 0, 0, 255]); // Black color
    let font_size = 20;
    let font: Vec<u8> = vec![0; font_size * font_size]; // Placeholder for character bitmap

    // Create a simple bitmap for the character
    // This would normally be replaced with proper font rendering
    for (i, pixel) in font.iter().enumerate() {
        let row = (i / font_size) as u32;
        let col = (i % font_size) as u32;
        if *pixel == 1 {
            img.put_pixel(x + col, y + row, char_color);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Dimensions of the GIF
    let width = 100;
    let height = 100;

    // Prepare the frames
    let mut frames = Vec::new();
    for i in 0..height {
        let mut frame = RgbaImage::new(width, height);

        // Draw the character at the current position
        let char_pos = (height - i) % height;
        draw_character(&mut frame, width / 2 - 10, char_pos, '∧');

        // Add the frame to the frames list
        frames.push(frame);
    }

    // Create the GIF file
    let mut output_file = File::create("character_animation.gif")?;
    let mut encoder = Encoder::new(&mut output_file, width as u16, height as u16, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;

    // Add each frame to the GIF
    for frame in frames {
        let mut gif_frame = Frame::from_rgba_speed(width as u16, height as u16, &mut frame.into_raw(), 10);
        gif_frame.delay = 10; // 100ms per frame
        encoder.write_frame(&gif_frame)?;
    }

    println!("GIF created successfully!");

    Ok(())
}*/


