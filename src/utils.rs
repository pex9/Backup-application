use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::{Pid, System};

use crate::backup::{Backupper, BackupperError};
use std::process::Command;
use std::env;

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

pub fn perform_backup(controller: Arc<(Mutex<bool>, Condvar)>) -> Result<(), BackupperError> {
    let (lock, cvar) = &*controller;

    let mut lk = lock.lock().unwrap();


    while *lk {
        lk = cvar.wait(lk).unwrap();
    }
    if !*lk {
        *lk = true;

        cvar.notify_all(); // Notify other thread es gui to stop
        let backupper = Backupper::new();
        backupper.perform_backup_with_stats()
    } else {
        Ok(())
    }
}

pub fn abort_backup(controller: Arc<(Mutex<bool>, Condvar)>) {
    let (lock, cvar) = &*controller;
    let mut lk = lock.lock().unwrap();
    while *lk {
        lk = cvar.wait(lk).unwrap();
    }
    *lk = true;
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
