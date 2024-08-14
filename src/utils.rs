use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::{Pid, System};

use crate::backup::{Backupper, BackupperError};
use std::process::Command;
use std::env;
use rodio::{Decoder, OutputStream, Sink};

pub fn start_monitor() {
    // Avvia il thread di monitoraggio CPU ( dovra essere una funzione)
    let sys = Arc::new(Mutex::new(System::new_all()));
    let sys_clone = Arc::clone(&sys);
    thread::spawn(move || {
        let pid = std::process::id();
        let pid = Pid::from_u32(pid); // Usa un metodo per creare Pid
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("config/cpu_usage.log")
            .unwrap();
        loop {
            let mut sys = sys_clone.lock().unwrap();
            sys.refresh_process(pid);
            if let Some(process) = sys.process(pid) {
                let cpu_usage = process.cpu_usage();
                let now = Local::now();
                let datetime_str = now.format("%d/%m/%Y %H:%M:%S").to_string();
                writeln!(file, "{} - CPU usage: {:.2}%", datetime_str, cpu_usage).expect("Failed to store logs");
            }
            thread::sleep(Duration::from_secs(120)); // Attendi 2 minuti
        }
    });
}

pub fn perform_backup(controller: Arc<Mutex<bool>>) -> Result<(), BackupperError> {    
    let mut lock = controller.lock().unwrap();
    if !*lock {
        *lock = true;

        play_sound("assets/backup_started.mp3");

        let backupper = Backupper::new();
        let backup_result = backupper.perform_backup_with_stats();

        match &backup_result {
            Ok(_) => play_sound("assets/backup_finished.mp3"),
            Err(e) => {
                play_sound("assets/backup_aborted.mp3");
                println!("Failed to perform backup: {:?}", e);
            }
        }

        backup_result

    } else {
        Ok(())
    }
}

pub fn abort_backup(controller: Arc<Mutex<bool>>) {
    let mut lock = controller.lock().unwrap();
    if !*lock {
        *lock = true;
        play_sound("assets/backup_cancelled.mp3");
    }
}

pub fn get_screensize() -> (u32, u32) {
    let output = Command::new(env::current_exe().unwrap().as_os_str())
        .arg("--screensize")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let dimensions: Vec<&str> = stdout.trim().split('-').collect();

    let width: u32 = dimensions[0].parse().unwrap_or(0);
    let height: u32 = dimensions[1].parse().unwrap_or(0);

    (width, height)
}

pub fn play_sound(path: &str) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    // Load a sound from a file, using a path relative to Cargo.toml
    let file = File::open(path).unwrap();
    let source = Decoder::new(BufReader::new(file)).unwrap();

    // Create a Sink to play the sound and wait until the audio is finished
    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(source);

    // Block the current thread until the sound has finished playing
    sink.sleep_until_end();
}
