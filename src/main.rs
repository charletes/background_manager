// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::sync::{Arc, Mutex};
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

    // Create tray icon
    // On Linux with ksni, IconSource::Resource can accept an absolute path
    let icon_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("tray.png");
    
    let icon = IconSource::Resource(Box::leak(
        icon_path.to_string_lossy().into_owned().into_boxed_str()
    ));

    let mut tray = TrayItem::new("Background Manager", icon)?;

    tray.add_label("Background Manager")?;

    // Toggle window visibility
    let ui_handle = ui.as_weak();
    let window_visible_clone = Arc::clone(&window_visible);
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

    // Quit application
    let ui_handle_quit = ui.as_weak();
    tray.add_menu_item("Quit", move || {
        if let Some(ui) = ui_handle_quit.upgrade() {
            ui.hide().unwrap();
        }
        std::process::exit(0);
    })?;

    // When window is closed with X, quit the application
    ui.window().on_close_requested(|| {
        std::process::exit(0);
    });

    ui.run()?;

    Ok(())
}
