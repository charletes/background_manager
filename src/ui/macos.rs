use tray_icon::{
    menu::{Menu, MenuEvent, MenuEventReceiver, MenuItem},
    TrayIcon, TrayIconBuilder,
};

use crate::ui::common::{load_and_invert_icon, on_close_requested};

slint::include_modules!();

pub fn run_macos() -> Result<(), slint::PlatformError> {
    let (tray_icon, menu_channel, show_item, exit_item) = create_tray_icon();
    let main_window = instantiate_ui();

    setup_event_handling(main_window.as_weak(), menu_channel, show_item, exit_item);

    // Keep the tray icon alive
    let _tray_icon = tray_icon;

    println!("Application is running. Tray icon is active.");

    let result = slint::run_event_loop_until_quit();

    println!("Application is closing.");

    result
}

fn create_tray_icon() -> (TrayIcon, MenuEventReceiver, MenuItem, MenuItem) {
    let (tray_icon, show_item, exit_item, show_id, exit_id) = create_tray_icon();
    println!("Tray icon has been set up.");

    let menu_channel = MenuEvent::receiver();

    (tray_icon, menu_channel.clone(), show_item, exit_item)
}

fn instantiate_ui() -> MainWindow {
    let main_window = MainWindow::new().unwrap();

    println!("Main window has been instantiated.");

    let window = main_window.window();

    let weak_window = main_window.as_weak();
    window.on_close_requested(move || on_close_requested(weak_window.clone()));

    main_window.on_quit(on_quit);

    main_window
}

fn setup_event_handling(
    main_window: slint::Weak<MainWindow>,
    menu_channel: MenuEventReceiver,
    show_item: MenuItem,
    exit_item: MenuItem,
) {
    main_window.unwrap().on_triggered(move || {
        handle_menu_events(main_window.clone(), &menu_channel, &show_item, &exit_item);
    });
}

fn handle_menu_events(
    main_window: slint::Weak<MainWindow>,
    menu_channel: &MenuEventReceiver,
    show_item: &MenuItem,
    exit_item: &MenuItem,
) {
    if let Ok(event) = menu_channel.try_recv() {
        if let Some(win) = main_window.upgrade() {
            if event.id == exit_item.id() {
                // win.invoke_quit();
                on_quit();
            } else if event.id == show_item.id() {
                win.show().unwrap();
                println!("Main window has been shown.");
            }
        }
    }
}

fn on_quit() {
    println!("Quit event received. Closing application.");
    slint::quit_event_loop().unwrap();
}
