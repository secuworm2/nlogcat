use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
}

fn default_font_family() -> String {
    "JetBrainsMono".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: Theme,
    pub font_size: f32,
    pub max_buffer_lines: usize,
    pub adb_path: Option<String>,
    pub auto_scroll: bool,
    pub window_width: f32,
    pub window_height: f32,
    #[serde(default = "default_font_family")]
    pub font_family: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            font_size: 12.0,
            max_buffer_lines: 300_000,
            adb_path: None,
            auto_scroll: true,
            window_width: 1280.0,
            window_height: 800.0,
            font_family: default_font_family(),
        }
    }
}

/// 설정 파일 경로를 반환한다: `%APPDATA%\nlogcat\settings.json`
#[must_use]
pub fn settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nlogcat")
        .join("settings.json")
}

/// 설정 파일을 읽어 반환한다. 파일이 없거나 파싱에 실패하면 기본값을 반환한다.
#[must_use]
pub fn load() -> AppSettings {
    let path = settings_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// 설정을 JSON 파일로 저장한다. 디렉토리가 없으면 자동으로 생성한다.
pub fn save(settings: &AppSettings) -> anyhow::Result<()> {
    let path = settings_path();
    let dir = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("invalid settings path"))?;
    std::fs::create_dir_all(dir)?;
    let json = serde_json::to_string_pretty(settings)?;
    std::fs::write(&path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{load, save, settings_path, AppSettings, Theme};

    #[test]
    fn default_values_are_correct() {
        let s = AppSettings::default();
        assert_eq!(s.theme, Theme::Dark);
        assert_eq!(s.font_size, 12.0_f32);
        assert_eq!(s.max_buffer_lines, 300_000);
        assert!(s.auto_scroll);
        assert_eq!(s.window_width, 1280.0_f32);
        assert_eq!(s.window_height, 800.0_f32);
        assert!(s.adb_path.is_none());
    }

    #[test]
    fn settings_path_ends_with_expected_components() {
        let path = settings_path();
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("nlogcat"), "path should contain 'nlogcat'");
        assert!(
            path_str.ends_with("settings.json"),
            "path should end with 'settings.json'"
        );
    }

    #[test]
    fn load_returns_default_without_panic() {
        // 파일 존재 여부와 관계없이 패닉이 발생하지 않아야 한다
        let _settings = load();
    }

    #[test]
    fn save_load_roundtrip() {
        // JSON 직렬화/역직렬화 사이클에서 값이 보존되어야 한다
        let original = AppSettings {
            font_size: 14.0,
            max_buffer_lines: 10_000,
            theme: Theme::Light,
            ..Default::default()
        };
        let json = serde_json::to_string_pretty(&original).unwrap();
        let loaded: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.theme, original.theme);
        assert_eq!(loaded.font_size, original.font_size);
        assert_eq!(loaded.max_buffer_lines, original.max_buffer_lines);
        assert_eq!(loaded.auto_scroll, original.auto_scroll);
    }

    #[test]
    fn save_and_load_from_disk() {
        save(&AppSettings::default()).expect("save should succeed");
        let loaded = load();
        assert_eq!(loaded.theme, Theme::Dark);
        assert_eq!(loaded.font_size, 12.0_f32);
    }
}
