mod gui; // Declare the module
mod utils;
use std::alloc::System;
use std::env;
use gui::MyApp; // Import the struct
use std::error::Error;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::time::Duration;
use winapi::um::winreg::HKEY_CURRENT_USER;
use winreg::RegKey;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // Aggiungi l'applicazione all'avvio di Windows release
    // add_to_startup()?;


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

/*
// windows release
fn add_to_startup() -> Result<(), Box<dyn std::error::Error>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let (key, _disp) = hkcu.create_subkey(path)?;

    // Ottieni il percorso dell'eseguibile corrente
    let exe_path: PathBuf = env::current_exe()?;
    let exe_path_str = exe_path.to_str().unwrap();

    // Aggiungi l'eseguibile al registro di Windows
    key.set_value("NomeApp", &exe_path_str)?;

    Ok(())
}
 */