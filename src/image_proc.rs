use photon_rs::PhotonImage;

pub fn fit_to_size(image: &PhotonImage, screen_size: (u32, u32)) -> PhotonImage {
    let img_width = image.get_width() as f64;
    let img_height = image.get_height() as f64;

    let screen_width = screen_size.0 as f64;
    let screen_height = screen_size.1 as f64;

    // Scale to fit (maintain aspect ratio, fit within screen)
    let fit_scale = (screen_width / img_width).min(screen_height / img_height);
    let fit_width = (img_width * fit_scale) as u32;
    let fit_height = (img_height * fit_scale) as u32;

    photon_rs::transform::resize(
        &image,
        fit_width,
        fit_height,
        photon_rs::transform::SamplingFilter::Lanczos3,
    )
}

pub fn fill_to_size(image: &PhotonImage, screen_size: (u32, u32)) -> PhotonImage {
    let img_width = image.get_width() as f64;
    let img_height = image.get_height() as f64;

    let screen_width = screen_size.0 as f64;
    let screen_height = screen_size.1 as f64;

    // Scale to fill (maintain aspect ratio, cover entire screen)
    let fill_scale = (screen_width / img_width).max(screen_height / img_height);
    let fill_width = (img_width * fill_scale) as u32;
    let fill_height = (img_height * fill_scale) as u32;

    let fill_img = photon_rs::transform::resize(
        image,
        fill_width,
        fill_height,
        photon_rs::transform::SamplingFilter::Lanczos3,
    );

    // Calculate center crop coordinates
    let center_x = fill_width as i32 / 2;
    let center_y = fill_height as i32 / 2;

    let top_left_x = center_x - (screen_width as i32 / 2);
    let top_left_y = center_y - (screen_height as i32 / 2);

    let bottom_right_x = top_left_x + screen_width as i32;
    let bottom_right_y = top_left_y + screen_height as i32;

    // Crop the fill image to screen size
    photon_rs::transform::crop(
        &fill_img,
        top_left_x as u32,
        top_left_y as u32,
        bottom_right_x as u32,
        bottom_right_y as u32,
    )
}

pub fn combine_fit_and_fill(
    fit_img: &PhotonImage,
    fill_img: &PhotonImage,
    screen_size: (u32, u32),
) -> PhotonImage {
    let mut fill_blur = fill_img.clone();
    let (screen_width, screen_height) = screen_size;
    photon_rs::conv::gaussian_blur(
        &mut fill_blur,
        (screen_width.max(screen_height) as f32 / 40.0) as i32,
    );
    // Paste the fit image centered on top of the blurred fill image
    let fit_width = fit_img.get_width();
    let fit_height = fit_img.get_height();
    let paste_x = (screen_width as u32 - fit_width) / 2;
    let paste_y = (screen_height as u32 - fit_height) / 2;
    photon_rs::multiple::watermark(&mut fill_blur, &fit_img, paste_x.into(), paste_y.into());

    fill_blur
}
