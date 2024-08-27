#![windows_subsystem = "windows"]
mod utils;
mod backup;
mod config;
mod config_gui;
mod confirm_gui;
mod error_gui;
mod launcher;

#[cfg(target_os = "macos")]
use daemonize::Daemonize;

use std::sync::{Arc, Mutex};
use std::{env, thread};

use config::CONFIG_FILE_PATH;
use config_gui::run_config_gui;
use confirm_gui::{run_confirm_gui, Choice};
use error_gui::run_error_gui;
use mouse::Mouse;
use utils::{abort_backup, get_screensize, perform_backup, get_abs_path};
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
    if !get_abs_path(CONFIG_FILE_PATH).exists() {
        run_error_gui("Error: First launch of the application: no configuration found. Please run the program with the --config flag to configure it.".to_string()).expect("Failed to run error gui");
        return;
    }
    #[cfg(target_os = "macos")]
    {
        Daemonize::new().start().expect("Failed to start system daemon");
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
                println!("Backup aborted: {:?}", e);
                abort_backup(controller);
                std::process::exit(0);
            }
        }
    });

    run_confirm_gui(sender, controller2).expect("Failed to run confirm gui");
}

fn main_configuration() {
    let conf_path = get_abs_path(CONFIG_FILE_PATH);
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
