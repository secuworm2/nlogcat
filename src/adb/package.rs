use std::collections::HashSet;
use std::os::windows::process::CommandExt;
use std::path::Path;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn fetch_user_packages(adb_path: &Path, serial: &str) -> HashSet<String> {
    let Ok(output) = std::process::Command::new(adb_path)
        .args(["-s", serial, "shell", "pm", "list", "packages", "-3"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    else {
        return HashSet::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.trim().strip_prefix("package:"))
        .map(|s| s.trim().to_string())
        .collect()
}
