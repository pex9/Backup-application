mod utils;

mod backup;
mod config;
mod config_gui;
mod confirm_gui;
mod launcher;


use std::thread::JoinHandle;
use std::{env, path::PathBuf, thread};

use config_gui::run_config_gui;
use confirm_gui::{run_confirm_gui, Choice};
use egui::epaint::tessellator::Path;
use mouse::Mouse;
use utils::perform_backup;
use std::fs::File;
use std::io::Write;

mod mouse;
mod sys;
pub mod types;

// run the confirmation gui
fn test() {
    let (sender, receiver) = std::sync::mpsc::channel();

    thread::spawn(move||{
        match receiver.recv() {
            Ok(choice) => {
                match choice {
                    Choice::Yes => {
                        print!("1");
                        std::process::exit(0);
                    }
                    Choice::No => {
                        print!("2");
                        std::process::exit(0);
                    }
                }
            }
            Err(_) => {
                print!("3");
                std::process::exit(0);
            }
        }
    });

    run_confirm_gui(sender);

}

/*
// run the configuration gui
fn main(){
    run_config_gui().unwrap();
}
*/
// use this if want to activate the program when load pc (windows tested, need to check also linux,macos)
/*fn main()
{
    auto_launch_app("/path/to/app");
}
*/
/* //use to run the back up function and write the corrispond log at the end of the operation

*/
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
            if mouse.rectangle_write(0, 0, 1430, 890).unwrap() {
                gesture_identified();
            }
        }
    }
}

fn gesture_identified() {
    let mut mouse = Mouse::new();
    thread::spawn(move || {
        if mouse.confirm().unwrap() {
            perform_backup();
        }
    });
    test();
}


fn main_configuration() {
    run_config_gui().unwrap();
}
