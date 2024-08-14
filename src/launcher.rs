use auto_launch::{AutoLaunch, AutoLaunchBuilder};
use std::env;

const APP_NAME: &str = "EmergencyBackup";

fn get_exe_path() -> String {
    env::current_exe()
        .expect("Failed to get current exe path")
        .to_str()
        .expect("Failed to convert exe path to string")
        .to_owned()
}

fn auto_launch_app() -> AutoLaunch {
    AutoLaunchBuilder::new()
        .set_app_name(APP_NAME)
        .set_app_path(&get_exe_path())
        .set_use_launch_agent(false)
        .build()
        .unwrap()
}

pub fn is_enabled() -> bool {
    let auto = auto_launch_app();
    auto.is_enabled().expect("Failed to check if auto launch is enabled")
}

pub fn enable() {
    let auto = auto_launch_app();
    auto.enable().expect("Failed to enable auto launch")
}

pub fn disable() {
    let auto = auto_launch_app();
    auto.disable().expect("Failed to disable auto launch")
}
