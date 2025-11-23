slint::include_modules!();

use slint::{ComponentHandle, Image, Rgba8Pixel, SharedPixelBuffer};
use std::path::Path;
use std::rc::Rc;

fn load_image_from_path(path: &Path) -> Option<Image> {
    let img = image::open(path).ok()?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(rgba.as_raw(), width, height);

    Some(Image::from_rgba8(buffer))
}

fn calculate_images_per_row(window_width: f32) -> usize {
    // Each image is 240px + spacing/padding (~260px total per image)
    let images_per_row = (window_width / 260.0).floor() as usize;
    images_per_row.max(1) // At least 1 image per row
}

fn update_image_layout(ui: &ImageGallery, images: &[Image], window_width: f32) {
    let images_per_row = calculate_images_per_row(window_width);
    let mut image_rows = Vec::new();

    for chunk in images.chunks(images_per_row) {
        let row_model = Rc::new(slint::VecModel::from(chunk.to_vec()));
        image_rows.push(slint::ModelRc::from(row_model));
    }

    let rows_model = Rc::new(slint::VecModel::from(image_rows));
    ui.set_image_rows(rows_model.into());
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = ImageGallery::new()?;

    // Load images from test_resources/thumbnails
    let thumbnails_path = Path::new("test_resources/thumbnails");
    let mut images = Vec::new();

    if let Ok(entries) = std::fs::read_dir(thumbnails_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("jpg") {
                if let Some(img) = load_image_from_path(&path) {
                    images.push(img);
                }
            }
        }
    }

    // Initial layout with current width
    let initial_width = ui.get_current_width();
    update_image_layout(&ui, &images, initial_width);

    // Set up callback for width changes
    let ui_weak = ui.as_weak();
    let images_clone = images.clone();
    ui.on_width_changed(move |new_width| {
        if let Some(ui) = ui_weak.upgrade() {
            update_image_layout(&ui, &images_clone, new_width);
        }
    });

    ui.run()
}
