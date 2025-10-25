#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};

#[cfg(target_os = "windows")]
unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    _lparam: LPARAM,
) -> BOOL {
    let mut monitor_info = MONITORINFOEXW {
        monitorInfo: windows::Win32::Graphics::Gdi::MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
            ..Default::default()
        },
        ..Default::default()
    };

    if GetMonitorInfoW(hmonitor, &mut monitor_info.monitorInfo as *mut _ as _).as_bool() {
        let rect = monitor_info.monitorInfo.rcMonitor;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        let device_name = String::from_utf16_lossy(&monitor_info.szDevice)
            .trim_end_matches('\0')
            .to_string();

        println!("Monitor: {}", device_name);
        println!("Resolution: {}x{}", width, height);
        println!("Position: ({}, {})", rect.left, rect.top);
        println!("---");
    }

    true.into()
}

#[cfg(not(target_os = "windows"))]
fn main() {
    println!("This example is only for Windows.");
}

#[cfg(target_os = "windows")]
fn main() {
    unsafe {
        EnumDisplayMonitors(None, None, Some(monitor_enum_proc), LPARAM(0));
    }
}
