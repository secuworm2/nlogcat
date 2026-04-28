use std::io::ErrorKind;
use std::path::Path;

use crate::adb::AdbError;
use crate::model::device::{Device, DeviceState};

pub fn list_devices(adb_path: &Path) -> Result<Vec<Device>, AdbError> {
    let output = std::process::Command::new(adb_path)
        .arg("devices")
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
pub fn get_device_model(adb_path: &Path, serial: &str) -> Option<String> {
    let output = std::process::Command::new(adb_path)
        .args(["-s", serial, "shell", "getprop", "ro.product.model"])
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
