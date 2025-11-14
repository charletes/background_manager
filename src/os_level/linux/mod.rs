use crate::os_level::MonitorInfo;

pub fn get_profile_info() -> Result<Vec<MonitorInfo>, String> {
    if is_wayland() {
        wayland::get_all()
    } else {
        xorg::get_all()
    }
}

use std::env::var_os;

mod wayland;
mod xorg;

fn is_wayland() -> bool {
    var_os("WAYLAND_DISPLAY")
        .or(var_os("XDG_SESSION_TYPE"))
        .is_some_and(|v| {
            v.to_str()
                .unwrap_or_default()
                .to_lowercase()
                .contains("wayland")
        })
}

pub fn set_background(absolute_path: &std::path::PathBuf, _desktop_num: i32) {
    let path = absolute_path.to_str().unwrap();
    println!("Path: {}", path);

    use more_wallpapers::Mode;

    let images = vec![path];
    more_wallpapers::set_wallpapers_from_vec(images, path, Mode::Crop).unwrap();
}
