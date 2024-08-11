use std::{error::Error, fs::File, io::BufReader, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct BackupConfig {
    pub source: String,
    pub destination: String,
    pub selected_extensions: Vec<String>,
    pub excluded_directories: Vec<String>,
    pub log_filename: String,
}

impl BackupConfig {
    pub fn new() -> Self {
        let mut app = Self {
            source: "".to_string(),
            destination: "".to_string(),
            selected_extensions: Vec::new(),
            excluded_directories: Vec::new(),
            log_filename: "backup_log.txt".to_string(),
        };

        // Load previously saved information
        app.load_info();
        app
    }

    pub fn save_info(&self) -> Result<(), Box<dyn Error>> {
        let path = "config/backup_info.json";
        let file = File::create(path)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }

    fn load_info(&mut self) {
        let path = "config/backup_info.json";
        if Path::new(path).exists() {
            let file = File::open(path).expect("Unable to open file");
            let reader = BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(loaded_info) => *self = loaded_info,
                Err(e) => {
                    eprintln!("Error loading JSON: {:?}", e);
                    // Handle JSON parsing errors by initializing with default values
                    self.selected_extensions = Vec::new();
                    self.excluded_directories = Vec::new();
                }
            }
        } else {
            // File does not exist; initialize with default values
            self.selected_extensions = Vec::new();
            self.excluded_directories = Vec::new();
        }
    }
}
