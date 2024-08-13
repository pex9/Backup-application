mod utils;

mod backup;
mod config;
mod config_gui;
mod confirm_gui;
mod launcher;


use std::sync::{Arc, Condvar, Mutex};
use std::{env, thread};
use std::process::exit;

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
    utils::start_monitor();
    let mut mouse = Mouse::new();
    let screensize = get_screensize();
    loop {
        let pos = mouse.get_position().unwrap();
        if pos.x == 0 && pos.y == 0 {
            if mouse.rectangle_write((screensize.0 as i32)-1, (screensize.1 as i32)-1).unwrap() {
                gesture_identified();

            }
        } else {
            thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}

fn gesture_identified() {
    let mut mouse = Mouse::new();
    let controller = Arc::new((Mutex::new(false), Condvar::new()));
    println!("{:?}", controller.0);
    let cont1 = Arc::clone(&controller);
    let cont2 = Arc::clone(&controller);
    let cont3 = Arc::clone(&controller);
    thread::spawn(move || {
        if mouse.confirm(cont1).unwrap() {
            if let Err(e) = perform_backup(cont2) {
                println!("Failed to perform backup: {:?}", e);
            }
        } else {
            abort_backup(cont3);
        }
    });

    // Start GUI confirmation
    gui_confirmation(controller);
}

fn gui_confirmation(controller: Arc<(Mutex<bool>, Condvar)>) {
    let (sender, receiver) = std::sync::mpsc::channel();
    let controller2 = Arc::clone(&controller);

    thread::spawn(move||{
        match receiver.recv() {
            Ok(choice) => {
                match choice {
                    Choice::Yes => {
                        if let Err(e) = perform_backup(controller) {
                            println!("Failed to perform backup: {:?}", e);
                        }
                    }
                    Choice::No => {
                        println!("Backup aborted 1");
                        abort_backup(controller);
                    },
                    Choice::CloseGui =>{
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

    run_confirm_gui(sender,controller2);

}

fn main_configuration() {
    run_config_gui().unwrap();
}

fn main_get_screensize() -> (u32, u32) {
    let event_loop= event_loop::EventLoop::new();
    let primary_monitor = event_loop.primary_monitor().unwrap();
    let physical_size = primary_monitor.size();
    let scale_factor = primary_monitor.scale_factor();
    let monitor_size = physical_size.to_logical(scale_factor);
    let width = monitor_size.width;
    let height = monitor_size.height;
    (width, height)
}
