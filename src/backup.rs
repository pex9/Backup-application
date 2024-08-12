use std::ffi::OsStr;
use std::path;
use std::sync::Mutex;

use cpu_time::ProcessTime;
use rebackup::{walker, WalkerConfig, WalkerErr, WalkerRule};

use crate::config::BackupConfig;
use std::fs::File;
use std::io::Write;

pub struct Backupper {
    backup_config: BackupConfig,
    walker_config: WalkerConfig,

    added_size: Mutex<u64>,
    removed_size: Mutex<u64>,
}

#[derive(Debug)]
pub enum BackupperError {
    BkpError(String),
    WalkerError(WalkerErr),
    IoError(Vec<std::io::Error>),
}

impl From<WalkerErr> for BackupperError {
    fn from(e: WalkerErr) -> Self {
        BackupperError::WalkerError(e)
    }
}

impl From<std::io::Error> for BackupperError {
    fn from(e: std::io::Error) -> Self {
        BackupperError::IoError(vec![e])
    }
}

impl Backupper {
    pub fn new() -> Self {
        let mut bkp = Self {
            backup_config: BackupConfig::new(),
            walker_config: WalkerConfig::new(Vec::new()),
            added_size: Mutex::new(0),
            removed_size: Mutex::new(0),
        };
        bkp.update_rules();
        bkp
    }

    pub fn update_rules(&mut self) {
        let mut rules = Vec::new();
        for ext in &self.backup_config.excluded_extensions {
            let e = ext.clone();
            rules.push(WalkerRule {
                name: "",
                description: None,
                only_for: Some(rebackup::WalkerItemType::File),
                matches: Box::new(move |x, _, _| x.extension() == Some(OsStr::new(e.as_str()))),
                action: Box::new(|_, _, _| Ok(rebackup::WalkerRuleResult::ExcludeItem)),
            });
        }
        for dir in &self.backup_config.excluded_directories {
            let d = dir.clone();
            rules.push(WalkerRule {
                name: "",
                description: None,
                only_for: Some(rebackup::WalkerItemType::Directory),
                matches: Box::new(move |x, _, _| {
                    let osstr = OsStr::new(d.as_str());
                    let p = path::Path::new(osstr);
                    if p.is_absolute() {
                        x.eq(p)
                    } else if d.contains(path::MAIN_SEPARATOR) {
                        x.ends_with(p)
                    } else {
                        x.file_name() == Some(osstr)
                    }
                }),
                action: Box::new(|_, _, _| Ok(rebackup::WalkerRuleResult::ExcludeItem)),
            });
        }
        self.walker_config = WalkerConfig::new(rules);
        self.walker_config.drop_empty_dirs = true;
    }

    fn get_target_files(&self) -> Result<Vec<path::PathBuf>, BackupperError> {
        let path = path::PathBuf::from(self.backup_config.source.clone());
        let mut data = walker::walk(&path, &self.walker_config)?;
        #[cfg(target_os = "windows")]
        {
            data = data.into_iter()
                .map(|x| {
                    x.strip_prefix(format!("\\\\?\\{}", self.backup_config.source).as_str())
                        .unwrap_or(&x)
                        .to_path_buf()
                })
                .collect();
        }
        Ok(data)
    }

    fn create_dst_path(&self, src: &path::PathBuf) -> path::PathBuf {
        let mut dst = path::PathBuf::from(self.backup_config.destination.clone());
        dst.push(src.strip_prefix(&self.backup_config.source).unwrap());
        dst
    }

    fn create_parent_if_not_exists(dst: &path::PathBuf) -> Result<(), std::io::Error> {
        if let Some(parent_dir) = dst.parent() {
            if !parent_dir.exists() {
                std::fs::create_dir_all(parent_dir)?;
            }
        }
        Ok(())
    }

    fn copy_file(&self, src: &path::PathBuf, dst: &path::PathBuf) -> Result<(), std::io::Error> {
        Self::create_parent_if_not_exists(dst)?;
        if dst.exists() {
            *self.removed_size.lock().unwrap() += std::fs::metadata(dst)?.len();
        }
        std::fs::copy(src, dst)?;
        *self.added_size.lock().unwrap() += std::fs::metadata(src)?.len();
        println!("File copied from {} to {}", src.display(), dst.display());
        Ok(())
    }

    fn copy_file_if_diffs(
        &self,
        src: &path::PathBuf,
        dst: &path::PathBuf,
    ) -> Result<(), std::io::Error> {
        let src_meta = std::fs::metadata(src)?;
        let dst_meta = std::fs::metadata(dst);
        match dst_meta {
            Ok(dm) => {
                if src_meta.len() != dm.len() {
                    self.copy_file(src, dst)?;
                } else {
                    let src_time = src_meta.modified()?;
                    let dst_time = dm.modified()?;
                    if src_time > dst_time {
                        self.copy_file(src, dst)?;
                    } else {
                        println!("File already up to date: {}", src.display());
                    }
                }
            }
            Err(_) => {
                self.copy_file(src, dst)?;
            }
        }
        Ok(())
    }

    pub fn perform_backup(&self) -> Result<(), BackupperError> {
        *self.added_size.lock().unwrap() = 0;
        *self.removed_size.lock().unwrap() = 0;

        let src = self.backup_config.source.clone();
        let dst = self.backup_config.destination.clone();

        if src.is_empty() || dst.is_empty() {
            return Err(BackupperError::BkpError(
                "Source or destination path is not set".to_string(),
            ));
        }

        if !path::Path::new(&src).is_dir() {
            return Err(BackupperError::BkpError(
                "Source is not a valid directory".to_string(),
            ));
        }

        let mut errors = Vec::new();

        let files = self.get_target_files()?;

        for file in files {
            let mut file = file;
            #[cfg(target_os = "windows")]
            {
                file = path::PathBuf::from(format!("{}{}{}", src, path::MAIN_SEPARATOR, file.display()));
            }
            let dst = self.create_dst_path(&file);
            match self.copy_file_if_diffs(&file, &dst) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error copying file {:?}: {}", file, e);
                    errors.push(e);
                }
            }
        }
        if errors.len() > 0 {
            return Err(BackupperError::IoError(errors));
        }

        Ok(())
    }

    fn create_log_file(&self) -> Result<File, std::io::Error> {
        let dst = path::PathBuf::from(self.backup_config.destination.clone())
            .join(&self.backup_config.log_filename);
        Self::create_parent_if_not_exists(&dst)?;
        File::create(dst)
    }

    fn write_log(
        &self,
        log_file: &mut File,
        start_clock_time: std::time::Instant,
        start_cpu_time: ProcessTime,
    ) -> Result<(), std::io::Error> {
        let added = *self.added_size.lock().unwrap();
        let removed = *self.removed_size.lock().unwrap();
        let duration = start_clock_time.elapsed();
        let cpu_duration = start_cpu_time.elapsed();

        writeln!(
            log_file,
            "Backup completed at: {} (total duration: {:.2?})",
            chrono::Utc::now(),
            duration
        )?;
        if added >= removed {
            let total_size = added - removed;
            writeln!(
                log_file,
                "Total size of files: {} bytes ({} added, {} removed)",
                total_size, added, removed
            )?;
        } else {
            let total_size = removed - added;
            writeln!(
                log_file,
                "Total size of files: -{} bytes ({} removed, {} added)",
                total_size, removed, added
            )?;
        }
        writeln!(log_file, "CPU time used: {:.2?}", cpu_duration)?;

        Ok(())
    }

    pub fn perform_backup_with_stats(&self) -> Result<(), BackupperError> {
        let start_clock_time = std::time::Instant::now();
        let start_cpu_time = ProcessTime::now();

        self.perform_backup()?;

        match self.create_log_file() {
            Ok(mut log_file) => {
                match self.write_log(&mut log_file, start_clock_time, start_cpu_time) {
                    Ok(_) => {
                        println!("Backup and log creation completed successfully.");
                    }
                    Err(e) => {
                        eprintln!("Error writing log file: {}", e);
                        return Err(BackupperError::IoError(vec![e]));
                    }
                }
            }
            Err(e) => {
                eprintln!("Error creating log file: {}", e);
                return Err(BackupperError::IoError(vec![e]));
            }
        }

        Ok(())
    }
}
