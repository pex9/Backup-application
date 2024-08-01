use std::env::current_dir;

use rebackup::{walker, WalkerConfig};

  pub struct Backupper {

  }

  impl Backupper {
    pub fn new() -> Self {
      let config = WalkerConfig::new(Vec::new());
      let path = current_dir().unwrap();
      let res = walker::walk(&path, &config);
      println!("{:?}", res);
      Self{}
    }
}
