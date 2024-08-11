mod utils;

mod backup;
mod config;
mod config_gui;
mod confirm_gui;
mod launcher;


use std::sync::{Arc, Mutex};
use std::{env, thread};

use config_gui::run_config_gui;
use confirm_gui::{run_confirm_gui, Choice};
use mouse::Mouse;
use utils::{abort_backup, perform_backup};

mod mouse;
mod sys;
pub mod types;

fn main() {
    if !launcher::is_enabled() {
        launcher::enable();
    }
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 && args[1] == "--config" {
        main_configuration();
    } else {
        main_background();
    }
}

fn main_background() {
    utils::start_monitor();
    let mut mouse = Mouse::new();
    loop {
        let pos = mouse.get_position().unwrap();
        if pos.x == 0 && pos.y == 0 {
            if mouse.rectangle_write(1430, 890).unwrap() {
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
    let cont1 = Arc::clone(&controller);
    thread::spawn(move || {
        if mouse.confirm().unwrap() {
            perform_backup(cont1).expect("Failed to perform backup");
        } else {
            abort_backup(cont1);
        }
    });
    gui_confirmation(controller);
}

fn gui_confirmation(mutex_controller: Arc<Mutex<bool>>) {
    let (sender, receiver) = std::sync::mpsc::channel();

    thread::spawn(move||{
        match receiver.recv() {
            Ok(choice) => {
                match choice {
                    Choice::Yes => {
                        perform_backup(mutex_controller).expect("Failed to perform backup");
                        //std::process::exit(0);
                    }
                    Choice::No => {
                        println!("Backup aborted 1");
                        abort_backup(mutex_controller);
                        //std::process::exit(0);
                    }
                }
            }
            Err(e) => {
                println!("Backup aborted 2: {:?}", e);
                abort_backup(mutex_controller);
                std::process::exit(0);
            }
        }
    });

    run_confirm_gui(sender);

}

fn main_configuration() {
    run_config_gui().unwrap();
}
