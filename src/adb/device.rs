use std::collections::HashMap;
use std::io::ErrorKind;
use std::os::windows::process::CommandExt;
use std::path::Path;

use crate::adb::AdbError;
use crate::model::device::{Device, DeviceState};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn list_devices(adb_path: &Path) -> Result<Vec<Device>, AdbError> {
    let output = std::process::Command::new(adb_path)
        .arg("devices")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                AdbError::NotFound {
                    path: adb_path.display().to_string(),
                }
            } else {
                AdbError::SpawnFailed(e)
            }
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let devices = stdout
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .filter_map(parse_device_line)
        .collect();

    Ok(devices)
}

fn parse_device_line(line: &str) -> Option<Device> {
    let (serial, state_str) = line.split_once('\t')?;
    let serial = serial.trim().to_string();
    if serial.is_empty() {
        return None;
    }

    let state = match state_str.trim() {
        "device" => DeviceState::Online,
        "offline" => DeviceState::Offline,
        "unauthorized" => DeviceState::Unauthorized,
        _ => return None,
    };

    Some(Device {
        serial,
        state,
        model: None,
    })
}

#[must_use]
pub fn query_pid_map(adb_path: &Path, serial: &str) -> HashMap<u32, String> {
    let Ok(output) = std::process::Command::new(adb_path)
        .args(["-s", serial, "shell", "ps", "-A"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    else {
        return HashMap::new();
    };
    parse_ps_output(&String::from_utf8_lossy(&output.stdout))
}

fn parse_ps_output(stdout: &str) -> HashMap<u32, String> {
    let mut map = HashMap::new();
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 || parts[1] == "PID" {
            continue;
        }
        let Ok(pid) = parts[1].parse::<u32>() else {
            continue;
        };
        if let Some(&name) = parts.last() {
            map.insert(pid, name.to_string());
        }
    }
    map
}

#[must_use]
pub fn get_device_model(adb_path: &Path, serial: &str) -> Option<String> {
    let output = std::process::Command::new(adb_path)
        .args(["-s", serial, "shell", "getprop", "ro.product.model"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let model = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if model.is_empty() {
        None
    } else {
        Some(model)
    }
}
