use crate::os_level::MonitorInfo;
use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};

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

pub fn set_background(_absolute_path: &std::path::PathBuf, _desktop_num: i32) {
    // Windows-specific implementation goes here
    unimplemented!()
}
