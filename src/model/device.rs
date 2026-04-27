#[derive(Debug, Clone, PartialEq)]
pub struct Device {
    pub serial: String,
    pub state: DeviceState,
    pub model: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceState {
    Online,
    Offline,
    Unauthorized,
}
