use std::fs;
use std::path::PathBuf;

use crate::mac;

pub fn change_background(absolute_path: &PathBuf, desktop_num: i32) -> Result<(), String> {
    let target_filename = format!("{}.jpg", desktop_num);

    // Check if the source file is already named n.jpg
    if let Some(filename) = absolute_path.file_name() {
        if filename.to_string_lossy() == target_filename {
            return Err(format!(
                "Error: Cannot use '{}' as source - it's the target filename for monitor {}",
                target_filename, desktop_num
            ));
        }
    }

    // Create path for the target file in the current directory
    let target_path = PathBuf::from(&target_filename);

    // Delete existing file if it exists
    if target_path.exists() {
        match fs::remove_file(&target_path) {
            Ok(_) => println!("  Removed existing '{}'", target_filename),
            Err(e) => {
                return Err(format!(
                    "  Monitor {} - Failed to remove existing '{}': {}",
                    desktop_num, target_filename, e
                ));
            }
        }
    }

    // Copy the file to n.jpg
    match fs::copy(&absolute_path, &target_path) {
        Ok(_) => println!("  Copied image to '{}'", target_filename),
        Err(e) => {
            return Err(format!(
                "  Monitor {} - Failed to copy file to '{}': {}",
                desktop_num, target_filename, e
            ));
        }
    }

    // Get absolute path of the copied file
    let copied_absolute_path = match target_path.canonicalize() {
        Ok(path) => path,
        Err(e) => {
            return Err(format!(
                "  Monitor {} - Failed to get absolute path of '{}': {}",
                desktop_num, target_filename, e
            ));
        }
    };

    // Set the picture
    mac::set_background(&copied_absolute_path, desktop_num);

    Ok(())
}
