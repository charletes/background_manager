// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::sync::{Arc, Mutex};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use tray_item::{IconSource, TrayItem};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    // Track window visibility state
    let window_visible = Arc::new(Mutex::new(true));

    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });

    // Platform-specific tray icon creation
    #[cfg(target_os = "linux")]
    {
        create_linux_tray(&ui, &window_visible)?;
    }

    #[cfg(target_os = "macos")]
    {
        create_macos_tray(&ui, &window_visible)?;
    }

    #[cfg(target_os = "windows")]
    {
        unimplemented!("Tray icon not implemented for Windows yet");
    }

    // Show the main window initially
    ui.show()?;

    eprintln!("[DEBUG] Starting event loop - will stay alive when window is hidden");

    // Use run_event_loop_until_quit instead of ui.run()
    // This keeps the event loop running even when all windows are hidden
    // Quit only when slint::quit_event_loop() is called (e.g., from Quit menu)
    slint::run_event_loop_until_quit()?;

    eprintln!("[DEBUG] Event loop exited");

    Ok(())
}

#[cfg(target_os = "linux")]
fn create_linux_tray(
    ui: &AppWindow,
    window_visible: &Arc<Mutex<bool>>,
) -> Result<(), Box<dyn Error>> {
    // Create tray icon
    // Select icon based on system theme
    let icon_filename = match dark_light::detect() {
        Ok(dark_light::Mode::Light) => "tray_light.png",
        _ => "tray_dark.png", // Default to dark icon for Dark mode or errors
    };

    let icon_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join(icon_filename);

    let icon = IconSource::Resource(Box::leak(
        icon_path.to_string_lossy().into_owned().into_boxed_str(),
    ));

    let mut tray = TrayItem::new("Background Manager", icon)?;

    tray.add_label("Background Manager")?;

    // Toggle window visibility
    let ui_handle = ui.as_weak();
    let window_visible_clone = Arc::clone(window_visible);
    tray.add_menu_item("Toggle Window", move || {
        if let Some(ui) = ui_handle.upgrade() {
            let mut visible = window_visible_clone.lock().unwrap();
            if *visible {
                ui.hide().unwrap();
                *visible = false;
            } else {
                ui.show().unwrap();
                *visible = true;
            }
        }
    })?;

    // "Hide" menu item to hide the window
    let ui_handle_hide = ui.as_weak();
    let window_visible_hide = Arc::clone(window_visible);
    tray.add_menu_item("Hide", move || {
        if let Some(ui) = ui_handle_hide.upgrade() {
            ui.hide().unwrap();
            let mut visible = window_visible_hide.lock().unwrap();
            *visible = false;
        }
    })?;

    // "Quit" menu item to actually quit the application
    tray.add_menu_item("Quit", move || {
        slint::quit_event_loop().unwrap();
    })?;

    // Keep tray alive by leaking it
    Box::leak(Box::new(tray));

    Ok(())
}

#[cfg(target_os = "macos")]
fn create_macos_tray(
    ui: &AppWindow,
    window_visible: &Arc<Mutex<bool>>,
) -> Result<(), Box<dyn Error>> {
    // Select icon based on system theme
    let icon_filename = match dark_light::detect() {
        Ok(dark_light::Mode::Light) => "tray_light.png",
        _ => "tray_dark.png", // Default to dark icon for Dark mode or errors
    };

    let icon_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join(icon_filename);

    println!("Using icon path: {:?}", icon_path);

    let icon = IconSource::Resource(Box::leak(
        icon_path.to_string_lossy().into_owned().into_boxed_str(),
    ));

    let mut tray = TrayItem::new("Background Manager", icon)?;

    tray.add_label("Background Manager")?;

    // Toggle window visibility
    let ui_handle = ui.as_weak();
    let window_visible_clone = Arc::clone(window_visible);
    tray.add_menu_item("Toggle Window", move || {
        if let Some(ui) = ui_handle.upgrade() {
            let mut visible = window_visible_clone.lock().unwrap();
            if *visible {
                ui.hide().unwrap();
                *visible = false;
            } else {
                ui.show().unwrap();
                *visible = true;
            }
        }
    })?;

    // Get inner tray for macOS-specific operations
    let mut inner = tray.inner_mut();
    inner.add_quit_item("Quit");
    inner.display();

    // Keep tray alive by leaking it
    Box::leak(Box::new(tray));

    Ok(())
}
