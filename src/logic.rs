use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::mac;

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
    let monitor_count = match mac::get_monitor_count() {
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

        let target_filename = format!("{}_{}.jpg", desktop_num, timestamp);

        // Check if the source file is already named n_timestamp.jpg (very unlikely but check anyway)
        if let Some(filename) = file_path.file_name() {
            if filename.to_string_lossy() == target_filename {
                eprintln!(
                    "Error: Cannot use '{}' as source - it's the target filename for monitor {}",
                    target_filename, desktop_num
                );
                continue;
            }
        }

        // Create path for the target file in the current directory
        let target_path = PathBuf::from(&target_filename);

        // Delete existing file if it exists (shouldn't happen with timestamp but just in case)
        if target_path.exists() {
            match fs::remove_file(&target_path) {
                Ok(_) => println!("  Removed existing '{}'", target_filename),
                Err(e) => {
                    eprintln!(
                        "  Monitor {} - Failed to remove existing '{}': {}",
                        desktop_num, target_filename, e
                    );
                    continue;
                }
            }
        }

        // Copy the file to n.jpg
        match fs::copy(&absolute_path, &target_path) {
            Ok(_) => println!("  Copied image to '{}'", target_filename),
            Err(e) => {
                eprintln!(
                    "  Monitor {} - Failed to copy file to '{}': {}",
                    desktop_num, target_filename, e
                );
                continue;
            }
        }

        // Get absolute path of the copied file
        let copied_absolute_path = match target_path.canonicalize() {
            Ok(path) => path,
            Err(e) => {
                eprintln!(
                    "  Monitor {} - Failed to get absolute path of '{}': {}",
                    desktop_num, target_filename, e
                );
                continue;
            }
        };

        // Set the picture
        mac::set_background(&copied_absolute_path, desktop_num);
    }
}

pub fn show_monitor_sizes() {
    let monitor_count = match mac::get_monitor_count() {
        Ok(count) => count,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    println!("Monitor sizes:");

    for desktop_num in 1..=monitor_count {
        match mac::get_monitor_size(desktop_num) {
            Ok((width, height)) => {
                println!("  Monitor {}: {}x{} pixels", desktop_num, width, height);
            }
            Err(e) => {
                eprintln!("  Monitor {} - Error: {}", desktop_num, e);
            }
        }
    }
}
