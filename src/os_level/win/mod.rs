use crate::os_level::MonitorInfo;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::{DesktopWallpaper, IDesktopWallpaper, DESKTOP_WALLPAPER_POSITION};

pub fn get_profile_info() -> Result<Vec<MonitorInfo>, String> {
    let mut monitors: Vec<MonitorInfo> = Vec::new();

    unsafe {
        // Use a raw pointer to pass the vector into the callback
        let monitors_ptr = &mut monitors as *mut Vec<MonitorInfo>;

        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(monitors_ptr as isize),
        );
    }

    if monitors.is_empty() {
        Err("No monitors found".to_string())
    } else {
        Ok(monitors)
    }
}

unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let monitors_ptr = lparam.0 as *mut Vec<MonitorInfo>;
    let monitors = &mut *monitors_ptr;

    let mut monitor_info = MONITORINFOEXW {
        monitorInfo: windows::Win32::Graphics::Gdi::MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
            ..Default::default()
        },
        ..Default::default()
    };

    if GetMonitorInfoW(hmonitor, &mut monitor_info.monitorInfo as *mut _ as _).as_bool() {
        let rect = monitor_info.monitorInfo.rcMonitor;
        let width = (rect.right - rect.left) as usize;
        let height = (rect.bottom - rect.top) as usize;

        let device_name = String::from_utf16_lossy(&monitor_info.szDevice)
            .trim_end_matches('\0')
            .to_string();

        let id = monitors.len() + 1;

        monitors.push(MonitorInfo {
            name: device_name,
            id,
            width,
            height,
        });
    }

    true.into()
}

pub fn set_background(absolute_path: &std::path::PathBuf, desktop_num: i32) {
    unsafe {
        // Initialize COM
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if hr.is_err() {
            eprintln!("Failed to initialize COM: {:?}", hr);
            return;
        }

        // Ensure COM is uninitialized when we exit
        let _com_guard = ComGuard;

        // Create the DesktopWallpaper instance
        let desktop_wallpaper: Result<IDesktopWallpaper, _> =
            CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL);

        let desktop_wallpaper = match desktop_wallpaper {
            Ok(dw) => dw,
            Err(e) => {
                eprintln!("Failed to create IDesktopWallpaper instance: {:?}", e);
                return;
            }
        };

        // Get the monitor ID for the specified desktop number
        let monitor_id = match get_monitor_id(&desktop_wallpaper, desktop_num) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };

        // Convert the path to a wide string
        let path_str = absolute_path.to_string_lossy();
        let mut path_wide: Vec<u16> = path_str.encode_utf16().collect();
        path_wide.push(0); // Null terminator

        // Set the wallpaper for the specific monitor
        if let Err(e) = desktop_wallpaper.SetWallpaper(
            PCWSTR::from_raw(monitor_id.as_ptr()),
            PCWSTR::from_raw(path_wide.as_ptr()),
        ) {
            eprintln!("Failed to set wallpaper: {:?}", e);
            return;
        }

        // Set the position to fill (DWPOS_FILL = 0)
        if let Err(e) = desktop_wallpaper.SetPosition(DESKTOP_WALLPAPER_POSITION(0)) {
            eprintln!("Failed to set wallpaper position: {:?}", e);
        }

        println!("  Monitor {} - Wallpaper set successfully", desktop_num);
    }
}

// Helper function to get the monitor ID string for a given desktop number
unsafe fn get_monitor_id(
    desktop_wallpaper: &IDesktopWallpaper,
    desktop_num: i32,
) -> Result<Vec<u16>, String> {
    let count = match desktop_wallpaper.GetMonitorDevicePathCount() {
        Ok(c) => c,
        Err(e) => return Err(format!("Failed to get monitor count: {:?}", e)),
    };

    if desktop_num < 1 || desktop_num > count as i32 {
        return Err(format!(
            "Monitor number {} is out of range (1-{})",
            desktop_num, count
        ));
    }

    // Get the monitor device path (0-indexed)
    let monitor_index = (desktop_num - 1) as u32;
    match desktop_wallpaper.GetMonitorDevicePathAt(monitor_index) {
        Ok(pwstr) => {
            // Convert PWSTR to Vec<u16>
            let mut len = 0;
            while *pwstr.0.add(len) != 0 {
                len += 1;
            }
            let mut result = Vec::with_capacity(len + 1);
            for i in 0..=len {
                result.push(*pwstr.0.add(i));
            }
            Ok(result)
        }
        Err(e) => Err(format!("Failed to get monitor device path: {:?}", e)),
    }
}

// Guard to ensure COM is properly uninitialized
struct ComGuard;

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
