#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    Android,
    Ios,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Device {
    pub serial: String,
    pub state: DeviceState,
    pub model: Option<String>,
    pub platform: Platform,
}

impl Device {
    /// 드롭다운에 표시할 레이블을 반환한다.
    #[must_use]
    pub fn display_label(&self) -> String {
        match self.platform {
            Platform::Android => self
                .model
                .as_deref()
                .map_or_else(|| format!("[Android] {}", self.serial), |m| format!("[Android] {m} ({})", self.serial)),
            Platform::Ios => self
                .model
                .as_deref()
                .map_or_else(|| format!("[iOS] {}", &self.serial[..self.serial.len().min(8)]), |n| format!("[iOS] {n}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceState {
    Online,
    Offline,
    Unauthorized,
}
