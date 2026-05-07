use std::path::{Path, PathBuf};

use std::os::windows::process::CommandExt;

use crate::ios::IosError;
use crate::model::{Device, DeviceState, Platform};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// `assets/bin/idevice_id.exe -l` 실행 결과로 연결된 iOS 기기 목록을 반환한다.
///
/// `bin_dir`을 찾을 수 없거나 실행 실패 시 빈 목록을 반환한다 (패닉 없음).
pub fn list_ios_devices(bin_dir: &Path) -> Vec<Device> {
    let exe = bin_dir.join("idevice_id.exe");
    let Ok(output) = std::process::Command::new(&exe)
        .arg("-l")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    else {
        return Vec::new();
    };

    let udids: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();

    udids
        .into_iter()
        .map(|udid| {
            let name = query_device_name(&exe, &udid);
            Device {
                serial: udid,
                state: DeviceState::Online,
                model: name,
                platform: Platform::Ios,
            }
        })
        .collect()
}

fn query_device_name(exe: &Path, udid: &str) -> Option<String> {
    let output = std::process::Command::new(exe)
        .args(["-n", udid])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    let name = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if name.is_empty() { None } else { Some(name) }
}

/// exe 옆 `assets/bin/` 또는 현재 디렉토리의 `assets/bin/`을 탐색한다.
///
/// # Errors
/// bin 디렉토리를 찾을 수 없으면 [`IosError::BinDirNotFound`]를 반환한다.
pub fn resolve_ios_bin_dir() -> Result<PathBuf, IosError> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("assets").join("bin");
            if candidate.is_dir() {
                return Ok(candidate);
            }
        }
    }
    let dev_candidate = PathBuf::from("assets/bin");
    if dev_candidate.is_dir() {
        return Ok(dev_candidate);
    }
    Err(IosError::BinDirNotFound)
}
