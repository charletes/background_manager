use std::path::PathBuf;

#[derive(Debug)]
pub struct MonitorInfo {
    pub name: String,
    pub id: usize,
    pub width: usize,
    pub height: usize,
}

#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "macos")]
pub fn get_monitor_count() -> Result<i32, String> {
    mac::get_monitor_count()
}

#[cfg(target_os = "windows")]
pub fn get_monitor_count() -> Result<i32, String> {
    win::get_monitor_count()
}

#[cfg(target_os = "macos")]
pub fn get_monitor_size(monitor_num: i32) -> Result<(i32, i32), String> {
    mac::get_monitor_size(monitor_num)
}

#[cfg(target_os = "windows")]
pub fn get_monitor_size(monitor_num: i32) -> Result<(i32, i32), String> {
    win::get_monitor_size(monitor_num)
}
#[cfg(target_os = "macos")]
pub fn set_background(absolute_path: &PathBuf, desktop_num: i32) {
    mac::set_background(absolute_path, desktop_num)
}
#[cfg(target_os = "windows")]
pub fn set_background(absolute_path: &PathBuf, desktop_num: i32) {
    win::set_background(absolute_path, desktop_num)
}
