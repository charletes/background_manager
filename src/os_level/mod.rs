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

fn get_profile_info() -> Result<Vec<MonitorInfo>, String> {
    #[cfg(target_os = "macos")]
    {
        return mac::get_profile_info();
    }
    #[cfg(target_os = "windows")]
    {
        return win::get_profile_info();
    }
}

pub fn get_monitor_count() -> Result<i32, String> {
    get_profile_info().map(|info| info.len() as i32)
}

pub fn get_monitor_size(monitor_num: i32) -> Result<(i32, i32), String> {
    let profile_info = get_profile_info()?;
    if monitor_num < 1 || (monitor_num as usize) > profile_info.len() {
        return Err(format!(
            "Monitor number {} is out of range (1-{})",
            monitor_num,
            profile_info.len()
        ));
    }

    let monitor_info = &profile_info[(monitor_num - 1) as usize];

    Ok((monitor_info.width as i32, monitor_info.height as i32))
}

#[cfg(target_os = "macos")]
pub fn set_background(absolute_path: &PathBuf, desktop_num: i32) {
    mac::set_background(absolute_path, desktop_num)
}
#[cfg(target_os = "windows")]
pub fn set_background(absolute_path: &PathBuf, desktop_num: i32) {
    win::set_background(absolute_path, desktop_num)
}
