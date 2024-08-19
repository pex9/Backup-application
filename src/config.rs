use std::{error::Error, fs::File, io::BufReader, path::Path};

use crate::launcher::{disable, enable, is_enabled};
use serde::{Deserialize, Serialize};

pub const CONFIG_FILE_PATH: &str = "config/backup_info.json";
pub const CPU_USAGE_LOG_PATH: &str = "config/cpu_usage.log";

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct BackupConfig {
    pub source: String,
    pub destination: String,
    pub excluded_extensions: Vec<String>,
    pub excluded_directories: Vec<String>,
    pub log_filename: String,
    #[serde(skip)]
    pub autostart_enabled: bool,
}

impl BackupConfig {
    pub fn new() -> Self {
        let mut app = Self {
            source: "".to_string(),
            destination: "".to_string(),
            excluded_extensions: Vec::new(),
            excluded_directories: Vec::new(),
            log_filename: "backup_log.txt".to_string(),
            autostart_enabled: false,
        };

        // Load previously saved information
        app.load_info();
        app
    }

    pub fn save_info(&self) -> Result<(), Box<dyn Error>> {
        if self.source.is_empty() || self.destination.is_empty() || self.log_filename.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Source, Destination, and Log Filename must be set before saving.",
            )));
        }
        let file = File::create(CONFIG_FILE_PATH)?;
        serde_json::to_writer(file, self)?;
        if self.autostart_enabled == true && is_enabled() == false {
            enable();
        } else if self.autostart_enabled == false && is_enabled() == true {
            disable();
        }
        Ok(())
    }

    fn load_info(&mut self) {
        if Path::new(CONFIG_FILE_PATH).exists() {
            let file = File::open(CONFIG_FILE_PATH).expect("Unable to open file");
            let reader = BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(loaded_info) => {
                    *self = {
                        self.autostart_enabled = is_enabled();
                        loaded_info
                    }
                }
                Err(e) => {
                    eprintln!("Error loading JSON: {:?}", e);
                    // Handle JSON parsing errors by initializing with default values
                    self.excluded_extensions = Vec::new();
                    self.excluded_directories = Vec::new();
                    self.autostart_enabled = is_enabled();
                }
            }
        } else {
            // File does not exist; initialize with default values
            self.excluded_extensions = Vec::new();
            self.excluded_directories = Vec::new();
            self.autostart_enabled = is_enabled();
        }
    }
}
