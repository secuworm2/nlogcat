use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const ICON_CANDIDATES: &[&str] = &[
    "res/mipmap-xxxhdpi-v4/ic_launcher.png",
    "res/mipmap-xxhdpi-v4/ic_launcher.png",
    "res/mipmap-xhdpi-v4/ic_launcher.png",
    "res/mipmap-hdpi-v4/ic_launcher.png",
    "res/mipmap-xxxhdpi-v4/ic_launcher.webp",
    "res/mipmap-xxhdpi-v4/ic_launcher.webp",
    "res/mipmap-xhdpi-v4/ic_launcher.webp",
    "res/mipmap-hdpi-v4/ic_launcher.webp",
    "res/mipmap-xxhdpi-v4/ic_launcher_round.png",
    "res/mipmap-xhdpi-v4/ic_launcher_round.png",
    "res/drawable-xxhdpi-v4/ic_launcher.png",
    "res/drawable/ic_launcher.png",
];

fn is_valid_image(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }
    if data.starts_with(b"\x89PNG") {
        return true;
    }
    &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP"
}

pub fn get_apk_path(adb_path: &Path, serial: &str, package: &str) -> Option<String> {
    let output = std::process::Command::new(adb_path)
        .args(["-s", serial, "shell", "pm", "path", package])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("package:"))
        .map(|s| s.trim().to_string())
}

pub fn extract_icon_bytes(adb_path: &Path, serial: &str, apk_path: &str) -> Option<Vec<u8>> {
    for candidate in ICON_CANDIDATES {
        let Ok(output) = std::process::Command::new(adb_path)
            .args(["-s", serial, "shell", "unzip", "-p", apk_path, candidate])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        else {
            continue;
        };
        if is_valid_image(&output.stdout) {
            return Some(output.stdout);
        }
    }
    None
}

fn icon_cache_path(package: &str) -> Option<PathBuf> {
    let dir = dirs::config_dir()?.join("nlogcat").join("icon_cache");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join(format!("{package}.img")))
}

pub fn load_icon_bytes(adb_path: &Path, serial: &str, package: &str) -> Option<Vec<u8>> {
    if let Some(path) = icon_cache_path(package) {
        if let Ok(cached) = std::fs::read(&path) {
            if is_valid_image(&cached) {
                return Some(cached);
            }
        }
    }

    let apk_path = get_apk_path(adb_path, serial, package)?;
    let bytes = extract_icon_bytes(adb_path, serial, &apk_path)?;

    if let Some(path) = icon_cache_path(package) {
        let _ = std::fs::write(path, &bytes);
    }

    Some(bytes)
}
