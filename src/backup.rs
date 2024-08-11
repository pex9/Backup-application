use std::sync::Mutex;
use std::path;
use std::ffi::OsStr;

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
  dim_overflow: Mutex<bool>,
}

#[derive(Debug)]
pub enum BackupperError {
  WalkerError(WalkerErr),
  IoError(Vec<std::io::Error>),
}

impl Backupper {
  pub fn new() -> Self {
    let mut bkp = Self{
      backup_config: BackupConfig::new(),
      walker_config: WalkerConfig::new(Vec::new()),
      added_size: Mutex::new(0),
      removed_size: Mutex::new(0),
      dim_overflow: Mutex::new(false),
    };
    bkp.update_rules();
    bkp
  }

  pub fn update_rules(&mut self) {
    let mut rules = Vec::new();
    for ext in &self.backup_config.selected_extensions {
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
  }

  fn get_target_files(&self) -> Result<Vec<path::PathBuf>, WalkerErr> {
    let path = path::PathBuf::from(self.backup_config.source.clone());
    walker::walk(&path, &self.walker_config)
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
      let dst_meta = std::fs::metadata(&dst)?;
      let dst_size = dst_meta.len();
      if self.removed_size.lock().unwrap().checked_add(dst_size).is_none() {
        eprintln!("Dimensions overflow");
        *self.dim_overflow.lock().unwrap() = true;
      }
    }
    std::fs::copy(src, dst)?;
    if self.added_size.lock().unwrap().checked_add(std::fs::metadata(src)?.len()).is_none() {
      eprintln!("Dimensions overflow");
      *self.dim_overflow.lock().unwrap() = true;
    }
    println!("File copied from {} to {}", src.display(), dst.display());
    Ok(())
  }

  fn copy_file_if_diffs(&self, src: &path::PathBuf, dst: &path::PathBuf) -> Result<(), std::io::Error> {
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
    *self.dim_overflow.lock().unwrap() = false;

    let mut errors = Vec::new();

    match self.get_target_files() {
      Ok(files) => {
        for file in files {
          let dst = self.create_dst_path(&file);
          match self.copy_file_if_diffs(&file, &dst) {
            Ok(_) => {
            }
            Err(e) => {
              eprintln!("Error copying file {:?}: {}", file, e);
              errors.push(e);
            }
          }
        }
        if errors.len() > 0 {
          return Err(BackupperError::IoError(errors));
        }
      }
      Err(e) => {
        eprintln!("Error: {}", e);
        return Err(BackupperError::WalkerError(e));
      }
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
    let total_size = added - removed;
    let duration = start_clock_time.elapsed();
    let cpu_duration = start_cpu_time.elapsed();

    writeln!(log_file, "Backup completed at: {} (total duration: {:.2?})", chrono::Utc::now(), duration)?;
    writeln!(log_file, "Total size of files: {} bytes ({} added, {} removed)", total_size, added, removed)?;
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
