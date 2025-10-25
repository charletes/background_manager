use crate::os_level::MonitorInfo;
use std::path::PathBuf;

pub fn get_profile_info() -> Result<Vec<MonitorInfo>, String> {
    // Windows-specific implementation goes here
    unimplemented!()
}

pub fn set_background(absolute_path: &std::path::PathBuf, desktop_num: i32) {
    // Windows-specific implementation goes here
    unimplemented!()
}
