use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::{Pid, System};

use crate::backup::{Backupper, BackupperError};

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

pub fn perform_backup(mutex_controller: Arc<Mutex<bool>>) -> Result<(), BackupperError> {
    let mut lk = mutex_controller.lock().unwrap();
    if !*lk {
        *lk = true;
        let backupper = Backupper::new();
        backupper.perform_backup_with_stats()
    } else {
        Ok(())
    }
}

pub fn abort_backup(mutex_controller: Arc<Mutex<bool>>) {
    let mut lk = mutex_controller.lock().unwrap();
    *lk = true;
}
