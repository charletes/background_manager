use regex;
use std::path::PathBuf;
use std::process::Command;

use crate::os_level::MonitorInfo;

fn get_profile_info() -> Result<Vec<MonitorInfo>, String> {
    // Run "system_profiler SPDisplaysDataType" and capture output
    let output = Command::new("system_profiler")
        .arg("SPDisplaysDataType")
        .output();

    let output = match output {
        Ok(output) => output,
        Err(e) => return Err(format!("Failed to execute system_profiler: {}", e)),
    };

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse output. Skip likes until "Displays:" is found
        let mut lines = stdout.lines();
        while let Some(line) = lines.next() {
            if line.trim() == "Displays:" {
                break;
            }
        }

        let remaining: Vec<&str> = lines.collect();

        let mut monitors: Vec<std::collections::HashMap<String, String>> = Vec::new();
        let mut current: Option<std::collections::HashMap<String, String>> = None;

        for raw in &remaining {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.ends_with(':') {
                // New monitor header
                if let Some(m) = current.take() {
                    monitors.push(m);
                }
                let name = trimmed.trim_end_matches(':').trim().to_string();
                let mut map = std::collections::HashMap::new();
                map.insert("name".to_string(), name);
                current = Some(map);
            } else {
                // Parameter line: split on first ':' into key and value
                if let Some(idx) = raw.find(':') {
                    let key = raw[..idx].trim().to_string();
                    let val = raw[idx + 1..].trim().to_string();
                    if let Some(m) = current.as_mut() {
                        m.insert(key, val);
                    } else {
                        // Parameter without explicit header — start anonymous monitor entry
                        let mut map = std::collections::HashMap::new();
                        map.insert(key, val);
                        current = Some(map);
                    }
                }
            }
        }

        if let Some(m) = current.take() {
            monitors.push(m);
        }

        let mut result: Vec<MonitorInfo> = Vec::new();

        for (id, m) in monitors.into_iter().enumerate() {
            if let Some(resolution) = m.get("Resolution") {
                {
                    // Match patterns like "3840 x 2160", "1920x1080", "2560×1600 Retina", etc.
                    // Accept optional spaces around 'x' and the unicode multiplication sign.
                    let re = regex::Regex::new(r"(?i)^\s*(\d+)\s*[x×]\s*(\d+)").unwrap();

                    if let Some(caps) = re.captures(resolution) {
                        let w = caps.get(1).and_then(|m| m.as_str().parse::<usize>().ok());
                        let h = caps.get(2).and_then(|m| m.as_str().parse::<usize>().ok());

                        match (w, h) {
                            (Some(width), Some(height)) => {
                                result.push(MonitorInfo {
                                    id: id + 1,
                                    name: m
                                        .get("name")
                                        .unwrap_or(&"Unknown".to_string())
                                        .to_string(),
                                    width,
                                    height,
                                });
                            }
                            _ => {
                                eprintln!(
                                    "  Could not parse numeric width/height from Resolution: {}",
                                    resolution
                                );
                            }
                        }
                    } else {
                        eprintln!(
                            "  Resolution string did not match expected pattern: {}",
                            resolution
                        );
                    }
                }
            }
        }

        Ok(result)
    } else {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
}

pub fn get_monitor_count() -> Result<i32, String> {
    let profile_info = get_profile_info()?;

    return Ok(profile_info.len() as i32);
}

pub fn get_monitor_size(monitor_num: i32) -> Result<(i32, i32), String> {
    let profile_info = get_profile_info()?;
    if monitor_num < 1 || (monitor_num as usize) > profile_info.len() {
        return Err(format!(
            "Monitor number {} is out of range (1-{})",
            monitor_num,
            profile_info.len()
        ));
    }

    let monitor_info = &profile_info[(monitor_num - 1) as usize];

    Ok((monitor_info.width as i32, monitor_info.height as i32))
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
