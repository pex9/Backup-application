mod config_gui; // Declare the module
mod utils;

use std::error::Error;

mod confirm_gui;

use confirm_gui::ConfirmGui;
use config_gui::run_config_gui;


use std::thread;
use confirm_gui::{Choice, run_confirm_gui};

/* run the confirmation gui
fn main() {
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
 */

// run the configuration gui
fn main(){
    run_config_gui().unwrap();
}

