use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::{Pid, System};

use crate::backup::{Backupper, BackupperError};
use auto_launch::AutoLaunchBuilder;

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
            .open("cpu_usage.log")
            .unwrap();
        loop {
            let mut sys = sys_clone.lock().unwrap();
            sys.refresh_process(pid);
            if let Some(process) = sys.process(pid) {
                let cpu_usage = process.cpu_usage();
                let now = Local::now();
                let datetime_str = now.format("%d/%m/%Y %H:%M:%S").to_string();
                writeln!(file, "{} - CPU usage: {:.2}%", datetime_str, cpu_usage);
            }
            thread::sleep(Duration::from_secs(120)); // Attendi 2 minuti
        }
    });
}

pub fn perform_backup() -> Result<(), BackupperError> {
    let backupper = Backupper::new();
    backupper.perform_backup_with_stats()
}

pub fn auto_launch_app(app_path: &str) {
    let app_name = "the-app";
    let args = &["--minimized"];
    let auto = AutoLaunchBuilder::new()
        .set_app_name(app_name)
        .set_app_path(app_path)
        .set_use_launch_agent(false)
        .set_args(args)
        .build()
        .unwrap();

    // enable the auto launch
    auto.enable().is_ok();
    println!("enabled: {}", auto.is_enabled().unwrap());

    // disable the auto launch
    auto.disable().is_ok();
    println!("enabled: {}", auto.is_enabled().unwrap());
}
