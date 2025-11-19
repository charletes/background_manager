use slint::ComponentHandle;

use crate::ui::MainWindow;

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
