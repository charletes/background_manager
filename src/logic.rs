use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::image_proc;
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
    let target_filename = format!("{}_{}.png", desktop_num, timestamp);
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

    // Process the image directly from source to target
    adjust_image(absolute_path, &target_path, desktop_num);

    // Get the absolute path of the created file
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

pub fn adjust_image(source_path: &PathBuf, target_path: &PathBuf, desktop_num: i32) {
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

    println!(
        "  Monitor {} - Processing image from '{}'",
        desktop_num,
        source_path.display()
    );

    // Open the image from source
    let img = match photon_rs::native::open_image(source_path.to_str().unwrap()) {
        Ok(image) => image,
        Err(e) => {
            eprintln!("  Monitor {} - Failed to open image: {}", desktop_num, e);
            return;
        }
    };

    // Scale to fit
    let fit_img = image_proc::fit_to_size(&img, (monitor_width, monitor_height));

    // Scale to fill
    let fill_img = image_proc::fill_to_size(&img, (monitor_width, monitor_height));

    // Combine fit and fill images
    let combined_img =
        image_proc::combine_fit_and_fill(&fit_img, &fill_img, (monitor_width, monitor_height));

    // Save the final composite image
    match photon_rs::native::save_image(combined_img, target_path.to_str().unwrap()) {
        Ok(_) => println!(
            "  Monitor {} - Successfully processed and saved image ({}x{})",
            desktop_num, monitor_width, monitor_height
        ),
        Err(e) => {
            eprintln!(
                "  Monitor {} - Failed to save processed image: {}",
                desktop_num, e
            );
        }
    }
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
