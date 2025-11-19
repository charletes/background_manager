use slint::ComponentHandle;

use crate::ui::MainWindow;

use tray_icon::{
    menu::{Menu, MenuId, MenuItem}, TrayIconBuilder,
};

pub fn on_close_requested(win: slint::Weak<MainWindow>) -> slint::CloseRequestResponse {
    if let Some(win) = win.upgrade() {
        win.hide().unwrap();
        println!("Main window has been hidden.");
    }
    slint::CloseRequestResponse::HideWindow
}

pub fn load_and_invert_icon(path: &str) -> (tray_icon::Icon, tray_icon::Icon) {
    let mut img = image::open(path)
        .expect("Failed to open icon path")
        .into_rgba8();

    let (width, height) = img.dimensions();
    let original_rgba = img.to_vec();
    let original_icon = tray_icon::Icon::from_rgba(original_rgba, width, height)
        .expect("Failed to create original icon");

    image::imageops::invert(&mut img);

    let inverted_rgba = img.into_raw();
    let inverted_icon = tray_icon::Icon::from_rgba(inverted_rgba, width, height)
        .expect("Failed to create inverted icon");

    (original_icon, inverted_icon)
}

pub fn create_tray_icon() -> (tray_icon::TrayIcon, MenuItem, MenuItem, MenuId, MenuId) {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tray_icon.png");
    let (dark_icon, light_icon) = load_and_invert_icon(path);

    let icon = match dark_light::detect() {
        Ok(dark_light::Mode::Dark) => light_icon,
        _ => dark_icon,
    };

    let tray_menu = Menu::new();
    let show_item = MenuItem::new("Show Window", true, None);
    let exit_item = MenuItem::new("Exit", true, None);

    let show_id = show_item.id().clone();
    let exit_id = exit_item.id().clone();

    tray_menu.append(&show_item).unwrap();
    tray_menu.append(&exit_item).unwrap();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Tray Test with Slint")
        .with_icon(icon)
        .build()
        .unwrap();

    (tray_icon, show_item, exit_item, show_id, exit_id)
}
