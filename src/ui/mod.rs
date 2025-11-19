#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

mod common;

#[cfg(target_os = "linux")]
pub use linux::MainWindow;

#[cfg(target_os = "macos")]
pub use macos::MainWindow;

pub fn run() -> Result<(), slint::PlatformError> {
    #[cfg(target_os = "macos")]
    {
        macos::run_macos()
    }

    #[cfg(target_os = "linux")]
    {
        linux::run_linux()
    }
}
