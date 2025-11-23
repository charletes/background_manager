use photon_rs::PhotonImage;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

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
        image,
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
    let paste_x = (screen_width - fit_width) / 2;
    let paste_y = (screen_height - fit_height) / 2;
    photon_rs::multiple::watermark(&mut fill_blur, fit_img, paste_x.into(), paste_y.into());

    fill_blur
}

/// Resize an image to a thumbnail with max dimension of 512px
pub fn resize_to_thumbnail(image: &PhotonImage) -> PhotonImage {
    let img_width = image.get_width();
    let img_height = image.get_height();
    let max_dimension = 512;

    // Calculate the larger dimension and scale
    let (new_width, new_height) = if img_width > img_height {
        let scale = max_dimension as f64 / img_width as f64;
        (max_dimension, (img_height as f64 * scale) as u32)
    } else {
        let scale = max_dimension as f64 / img_height as f64;
        ((img_width as f64 * scale) as u32, max_dimension)
    };

    photon_rs::transform::resize(
        image,
        new_width,
        new_height,
        photon_rs::transform::SamplingFilter::Lanczos3,
    )
}

/// Check if thumbnail needs to be regenerated based on file modification times
pub fn needs_thumbnail_update(source_path: &Path, thumbnail_path: &Path) -> bool {
    // If thumbnail doesn't exist, we need to create it
    if !thumbnail_path.exists() {
        return true;
    }

    // Get modification times
    let source_modified = match fs::metadata(source_path).and_then(|m| m.modified()) {
        Ok(time) => time,
        Err(_) => return true, // If we can't read source metadata, regenerate to be safe
    };

    let thumbnail_modified = match fs::metadata(thumbnail_path).and_then(|m| m.modified()) {
        Ok(time) => time,
        Err(_) => return true, // If we can't read thumbnail metadata, regenerate
    };

    // Regenerate if source is newer than thumbnail
    source_modified > thumbnail_modified
}

/// Generate thumbnails for all JPG files in test_resources folder
pub fn generate_thumbnails() -> Result<(), Box<dyn std::error::Error>> {
    let source_dir = Path::new("test_resources");
    let thumbnail_dir = source_dir.join("thumbnails");

    // Create thumbnails directory if it doesn't exist
    fs::create_dir_all(&thumbnail_dir)?;

    // Collect all JPG file paths first
    let jpg_files: Vec<_> = fs::read_dir(source_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            if !path.is_file() {
                return false;
            }
            let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            extension.to_lowercase() == "jpg" || extension.to_lowercase() == "jpeg"
        })
        .collect();

    // Use a mutex to collect errors
    let errors = Mutex::new(Vec::new());

    // Process files in parallel
    jpg_files.par_iter().for_each(|path| {
        // Get the filename
        let filename = match path.file_name() {
            Some(name) => name,
            None => return,
        };

        let thumbnail_path = thumbnail_dir.join(filename);

        // Check if we need to update the thumbnail
        if !needs_thumbnail_update(path, &thumbnail_path) {
            return; // Skip this file, thumbnail is up to date
        }

        // Process the thumbnail
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            // If thumbnail exists and is outdated, delete it
            if thumbnail_path.exists() {
                fs::remove_file(&thumbnail_path)?;
            }

            // Load the image
            let img = photon_rs::native::open_image(path.to_str().unwrap())?;

            // Resize to thumbnail
            let thumbnail = resize_to_thumbnail(&img);

            // Save the thumbnail
            photon_rs::native::save_image(thumbnail, thumbnail_path.to_str().unwrap())?;

            println!("Converted: {}", path.display());
            Ok(())
        })();

        if let Err(e) = result {
            errors
                .lock()
                .unwrap()
                .push(format!("{}: {}", path.display(), e));
        }
    });

    // Check if there were any errors
    let errors = errors.into_inner().unwrap();
    if !errors.is_empty() {
        return Err(format!("Errors occurred:\n{}", errors.join("\n")).into());
    }

    Ok(())
}
