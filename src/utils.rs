use std::fs::{File, OpenOptions};
use std::sync::{Arc, Mutex};
use std::{fs, io, thread};
use std::time::{Duration, Instant};
use sysinfo::{Pid, System};
use std::io::{Write};
use std::path::Path;
use chrono::{Local, Utc};

use auto_launch::AutoLaunch;
use crate::config_gui::MyApp;


pub fn start_monitor() {
    // Avvia il thread di monitoraggio CPU ( dovra essere una funzione)
    let sys = Arc::new(Mutex::new(System::new_all()));
    let sys_clone = Arc::clone(&sys);
    thread::spawn(move || {
        let pid = std::process::id();
        let pid = Pid::from_u32(pid); // Usa un metodo per creare Pid
        let mut file = OpenOptions::new().append(true).create(true).open("cpu_usage.log").unwrap();
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

pub fn perform_backup(back_up: MyApp) -> io::Result<()> {
    // Paths to source and destination (replace these with actual paths)

    let source_dir = Path::new(&back_up.source);
    let destination_dir = Path::new(&back_up.destination);
    // Create the destination directory if it doesn't exist
    if !destination_dir.exists() {
        fs::create_dir_all(destination_dir)?;
    }

    // Start timing the CPU usage
    let start_time = Instant::now();

    // Perform the backup and calculate the total size to be done
    let total_size = 0;//backup(source_dir, destination_dir)?;

    // Calculate the CPU time used
    let duration = start_time.elapsed();

    // Create the log file
    let log_file_path = destination_dir.join("backup_log.txt");
    let mut log_file = File::create(log_file_path)?;

    // Write the log information
    writeln!(log_file, "Backup completed at: {}", Utc::now())?;
    writeln!(log_file, "Total size of files: {} bytes", total_size)?;
    writeln!(log_file, "CPU time used: {:.2?}", duration)?;

    println!("Backup and log creation completed successfully.");

    Ok(())
}


pub fn auto_launch_windows() {
    let app_name = "the-app";
    let app_path = "C:\\path\\to\\the-app.exe";
    let args = &["--minimized"];
    let auto = AutoLaunch::new(app_name, app_path, args);

    // enable the auto launch
    auto.enable().is_ok();
    auto.is_enabled().unwrap();

    // disable the auto launch
    //auto.disable().is_ok();
    //auto.is_enabled().unwrap();
}

pub fn auto_launch_mac_os() {
    let app_name = "the-app";
    let app_path = "/path/to/the-app.app";
    let args = &["--minimized"];
    let auto = AutoLaunch::new(app_name, app_path, args);

    // enable the auto launch
    auto.enable().is_ok();
    auto.is_enabled().unwrap();

    // disable the auto launch
    auto.disable().is_ok();
    auto.is_enabled().unwrap();
}

pub fn auto_launch_linux()
{
    let app_name = "the-app";
    let app_path = "/path/to/the-app";
    let args = &["--minimized"];
    let auto = AutoLaunch::new(app_name, app_path,args);

    // enable the auto launch
    auto.enable().is_ok();
    auto.is_enabled().unwrap();

    // disable the auto launch
    auto.disable().is_ok();
    auto.is_enabled().unwrap();
}
