use tray_icon::{
    menu::{Menu, MenuItem, MenuEventReceiver},
    TrayIcon, TrayIconBuilder,
};

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
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

#[cfg(target_os = "macos")]
fn create_tray_icon() -> (TrayIcon, MenuEventReceiver, MenuItem, MenuItem) {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tray_icon.png");
    let (dark_icon, light_icon) = load_and_invert_icon(path);

    let icon = match dark_light::detect() {
        Ok(dark_light::Mode::Dark) => light_icon,
        _ => dark_icon,
    };

    let tray_menu = Menu::new();
    let show_item = MenuItem::new("Show Window", true, None);
    tray_menu.append(&show_item).unwrap();
    let exit_item = MenuItem::new("Exit", true, None);
    tray_menu.append(&exit_item).unwrap();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Tray Test with Slint")
        .with_icon(icon)
        .build()
        .unwrap();

    println!("Tray icon has been set up.");

    let menu_channel = tray_icon::menu::MenuEvent::receiver();

    (tray_icon, menu_channel.clone(), show_item, exit_item)
}

#[cfg(not(target_os = "macos"))]
fn create_tray_icon() -> (TrayIcon, MenuEventReceiver, MenuItem, MenuItem) {
    unimplemented!("Tray icon is only supported on macOS in this application.");
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
        handle_menu_events(
            main_window.clone(),
            &menu_channel,
            &show_item,
            &exit_item,
        );
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
                win.invoke_quit();
            } else if event.id == show_item.id() {
                win.show().unwrap();
                println!("Main window has been shown.");
            }
        }
    }
}

fn on_close_requested(win: slint::Weak<MainWindow>) -> slint::CloseRequestResponse {
    if let Some(win) = win.upgrade() {
        win.hide().unwrap();
        println!("Main window has been hidden.");
    }
    slint::CloseRequestResponse::HideWindow
}

fn on_quit() {
    println!("Quit event received. Closing application.");
    slint::quit_event_loop().unwrap();
}

fn load_and_invert_icon(path: &str) -> (tray_icon::Icon, tray_icon::Icon) {
    let mut img = image::open(path)
        .expect("Failed to open icon path")
        .into_rgba8();

    let (width, height) = img.dimensions();
    let original_rgba = img.to_vec();
    let original_icon =
        tray_icon::Icon::from_rgba(original_rgba, width, height).expect("Failed to create original icon");

    image::imageops::invert(&mut img);

    let inverted_rgba = img.into_raw();
    let inverted_icon =
        tray_icon::Icon::from_rgba(inverted_rgba, width, height).expect("Failed to create inverted icon");

    (original_icon, inverted_icon)
}