use std::fs;
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::os_level;

pub fn change_background(args: &[String]) {
    // Check if we have the required file parameter
    if args.is_empty() {
        eprintln!("Error: -change requires a file name parameter");
        eprintln!("Usage:");
        eprintln!("  background_manager -change <file>           Set image on all monitors");
        eprintln!("  background_manager -change <file> <monitor> Set image on specific monitor");
        return;
    }

    let file_path = PathBuf::from(&args[0]);

    // Verify the file exists
    if !file_path.exists() {
        eprintln!("Error: File '{}' does not exist", file_path.display());
        return;
    }

    if !file_path.is_file() {
        eprintln!("Error: '{}' is not a file", file_path.display());
        return;
    }

    // Get absolute path
    let absolute_path = match file_path.canonicalize() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to get absolute path: {}", e);
            return;
        }
    };

    println!("Selected: {}", file_path.display());

    // Get the number of monitors
    let monitor_count = match os_level::get_monitor_count() {
        Ok(count) => count,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    // Check if a specific monitor was specified
    let target_monitors: Vec<i32> = if args.len() >= 2 {
        // Specific monitor provided
        match args[1].parse::<i32>() {
            Ok(monitor_num) => {
                if monitor_num < 1 || monitor_num > monitor_count {
                    eprintln!(
                        "Error: Monitor number must be between 1 and {}",
                        monitor_count
                    );
                    return;
                }
                vec![monitor_num]
            }
            Err(_) => {
                eprintln!("Error: Invalid monitor number '{}'", args[1]);
                return;
            }
        }
    } else {
        // No specific monitor, set on all monitors
        (1..=monitor_count).collect()
    };

    if target_monitors.len() == 1 {
        println!("Setting background on monitor {}...", target_monitors[0]);
    } else {
        println!(
            "Setting background on {} monitor(s)...",
            target_monitors.len()
        );
    }

    for desktop_num in target_monitors {
        // Get current time in seconds since UNIX epoch
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let copied_absolute_path =
            match create_target_file(&file_path, &absolute_path, desktop_num, timestamp) {
                Some(value) => value,
                None => continue,
            };

        // Set the picture
        os_level::set_background(&copied_absolute_path, desktop_num);
    }
}

fn create_target_file(
    file_path: &PathBuf,
    absolute_path: &PathBuf,
    desktop_num: i32,
    timestamp: u64,
) -> Option<PathBuf> {
    // Define the target filename
    let target_filename = format!("{}_{}.jpg", desktop_num, timestamp);
    if let Some(filename) = file_path.file_name() {
        if filename.to_string_lossy() == target_filename {
            eprintln!(
                "Error: Cannot use '{}' as source - it's the target filename for monitor {}",
                target_filename, desktop_num
            );
            return None;
        }
    }

    // Using the new name, set the path.
    let target_path = PathBuf::from(&target_filename);
    if target_path.exists() {
        match fs::remove_file(&target_path) {
            Ok(_) => println!("  Removed existing '{}'", target_filename),
            Err(e) => {
                eprintln!(
                    "  Monitor {} - Failed to remove existing '{}': {}",
                    desktop_num, target_filename, e
                );
                return None;
            }
        }
    }

    // Copy file to destination
    match fs::copy(absolute_path, &target_path) {
        Ok(_) => println!("  Copied image to '{}'", target_filename),
        Err(e) => {
            eprintln!(
                "  Monitor {} - Failed to copy file to '{}': {}",
                desktop_num, target_filename, e
            );
            return None;
        }
    }

    // Modify destination file
    adjust_image(&target_path, desktop_num);

    // Get the absolute path of the copied file
    let copied_absolute_path = match target_path.canonicalize() {
        Ok(path) => path,
        Err(e) => {
            eprintln!(
                "  Monitor {} - Failed to get absolute path of '{}': {}",
                desktop_num, target_filename, e
            );
            return None;
        }
    };
    Some(copied_absolute_path)
}

pub fn adjust_image(target_path: &PathBuf, desktop_num: i32) {
    // Get the monitor size
    let (monitor_width, monitor_height) = match os_level::get_monitor_size(desktop_num) {
        Ok(size) => size,
        Err(e) => {
            eprintln!(
                "  Monitor {} - Failed to get monitor size: {}",
                desktop_num, e
            );
            return;
        }
    };

    // Open the image
    let mut img = match photon_rs::native::open_image(target_path.to_str().unwrap()) {
        Ok(image) => image,
        Err(e) => {
            eprintln!("  Monitor {} - Failed to open image: {}", desktop_num, e);
            return;
        }
    };

    let img_width = img.get_width();
    let img_height = img.get_height();

    // Check if resizing is needed
    if img_width != monitor_width || img_height != monitor_height {
        println!(
            "  Monitor {} - Image size {}x{} differs from monitor {}x{}, scaling...",
            desktop_num, img_width, img_height, monitor_width, monitor_height
        );

        // Calculate scale factor to ensure both dimensions are at least monitor size
        let scale_x = monitor_width as f32 / img_width as f32;
        let scale_y = monitor_height as f32 / img_height as f32;
        let scale = scale_x.max(scale_y);

        let new_width = (img_width as f32 * scale) as u32;
        let new_height = (img_height as f32 * scale) as u32;

        // Resize the image
        img = photon_rs::transform::resize(
            &img,
            new_width,
            new_height,
            photon_rs::transform::SamplingFilter::Lanczos3,
        );

        // Crop the image to monitor size, centered
        let crop_x = (new_width - monitor_width) / 2;
        let crop_y = (new_height - monitor_height) / 2;
        img = photon_rs::transform::crop(&img, crop_x, crop_y, monitor_width, monitor_height);

        // Save the resized image
        match photon_rs::native::save_image(img, target_path.to_str().unwrap()) {
            Ok(_) => println!(
                "  Monitor {} - Resized image to {}x{}",
                desktop_num, new_width, new_height
            ),
            Err(e) => {
                eprintln!(
                    "  Monitor {} - Failed to save resized image: {}",
                    desktop_num, e
                );
            }
        }
    } else {
        println!(
            "  Monitor {} - Image size matches monitor, no scaling needed",
            desktop_num
        );
    }

    // Placeholder for image adjustment logic
    // For example, resizing or cropping the image to fit the monitor's resolution
    println!(
        "  Monitor {} - Adjusting image at '{}'",
        desktop_num,
        target_path.display()
    );
    // Actual image processing code would go here
}

pub fn show_monitor_sizes() {
    let monitor_count = match os_level::get_monitor_count() {
        Ok(count) => count,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    println!("Monitor sizes:");

    for mon_number in 1..=monitor_count {
        match os_level::get_monitor_size(mon_number) {
            Ok((width, height)) => {
                println!("  Monitor {}: {}x{} pixels", mon_number, width, height);
            }
            Err(e) => {
                eprintln!("  Monitor {} - Error: {}", mon_number, e);
            }
        }
    }
}
