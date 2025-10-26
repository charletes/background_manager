fn main() {
    let original_file = r"D:\Projects\background_manager\test_resources\test_c.jpg";

    // Get screen dimensions
    let (screen_width, screen_height) = (1920.0_f64, 1080.0_f64);

    // Open the image
    let mut img = photon_rs::native::open_image(original_file).expect("Failed to open image");
    let img_width = img.get_width() as f64;
    let img_height = img.get_height() as f64;

    // Scale to fit (maintain aspect ratio, fit within screen)
    let fit_scale = (screen_width / img_width).min(screen_height / img_height);
    let fit_width = (img_width * fit_scale) as u32;
    let fit_height = (img_height * fit_scale) as u32;
    let fit_img = photon_rs::transform::resize(
        &img,
        fit_width,
        fit_height,
        photon_rs::transform::SamplingFilter::Lanczos3,
    );

    photon_rs::native::save_image(fit_img.clone(), "fit.jpg").expect("Failed to save fit.jpg");

    // Scale to fill (maintain aspect ratio, cover entire screen)
    let fill_scale = (screen_width / img_width).max(screen_height / img_height);
    let fill_width = (img_width * fill_scale) as u32;
    let fill_height = (img_height * fill_scale) as u32;
    let fill_img = photon_rs::transform::resize(
        &img,
        fill_width,
        fill_height,
        photon_rs::transform::SamplingFilter::Lanczos3,
    );

    let center_x = fill_width as i32 / 2;
    let center_y = fill_height as i32 / 2;

    let (top_left_x, top_left_y) = (
        center_x - (screen_width as i32 / 2),
        center_y - (screen_height as i32 / 2),
    );

    let (bottom_right_x, bottom_right_y) = (
        top_left_x + (screen_width as i32),
        top_left_y + (screen_height as i32),
    );

    // Crop expects (x, y, width, height) not bottom-right coordinates
    let mut fill_crop_img = photon_rs::transform::crop(
        &fill_img,
        top_left_x as u32,
        top_left_y as u32,
        bottom_right_x as u32,
        bottom_right_y as u32,
    );

    // gaussian_blur expects an f32 sigma, not an integer
    photon_rs::conv::gaussian_blur(&mut fill_crop_img, (screen_width as f32 / 40.0) as i32);

    // Paste the fit image centered on top of the fill image
    let paste_x = (screen_width as u32 - fit_width) / 2;
    let paste_y = (screen_height as u32 - fit_height) / 2;
    photon_rs::multiple::watermark(&mut fill_crop_img, &fit_img, paste_x.into(), paste_y.into());

    photon_rs::native::save_image(fill_crop_img, "final.jpg").expect("Failed to save final.jpg");
}
