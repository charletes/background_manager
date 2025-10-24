use std::path::PathBuf;
use std::process::Command;

pub fn get_monitor_count() -> Result<i32, String> {
    let script = r#"
        tell application "System Events"
            count of desktops
        end tell
    "#;

    match Command::new("osascript").arg("-e").arg(script).output() {
        Ok(output) => {
            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout);
                let count = result
                    .trim()
                    .parse::<i32>()
                    .map_err(|e| format!("Failed to parse monitor count: {}", e))?;
                Ok(count)
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(format!("Error querying monitors: {}", error))
            }
        }
        Err(e) => Err(format!("Failed to execute AppleScript: {}", e)),
    }
}

pub fn get_monitor_size(monitor_num: i32) -> Result<(i32, i32), String> {
    let script = r#"tell application "Finder"
        set screenBounds to bounds of window of desktop
        set screenWidth to item 3 of screenBounds
        set screenHeight to item 4 of screenBounds
        return (screenWidth as string) & "," & (screenHeight as string)
    end tell"#;

    match Command::new("osascript").arg("-e").arg(&script).output() {
        Ok(output) => {
            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = result.trim().split(',').collect();

                if parts.len() != 2 {
                    return Err(format!("Invalid size format for monitor {}", monitor_num));
                }

                let width = parts[0]
                    .parse::<i32>()
                    .map_err(|e| format!("Failed to parse width: {}", e))?;
                let height = parts[1]
                    .parse::<i32>()
                    .map_err(|e| format!("Failed to parse height: {}", e))?;

                Ok((width, height))
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(format!("Error querying monitor size: {}", error))
            }
        }
        Err(e) => Err(format!("Failed to execute AppleScript: {}", e)),
    }
}

pub fn set_background(absolute_path: &PathBuf, desktop_num: i32) {
    let set_picture_script = format!(
        r#"tell application "System Events"
                set picture of desktop {} to "{}"
            end tell"#,
        desktop_num,
        absolute_path.display()
    );

    match Command::new("osascript")
        .arg("-e")
        .arg(&set_picture_script)
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                println!("  Monitor {} - Background set successfully", desktop_num);
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                eprintln!("  Monitor {} - Error: {}", desktop_num, error);
            }
        }
        Err(e) => {
            eprintln!("  Monitor {} - Failed to execute: {}", desktop_num, e);
        }
    }
}
