use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Verbose,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
    Unknown,
}

impl LogLevel {
    #[must_use]
    pub fn from_char(c: char) -> Self {
        match c {
            'V' => Self::Verbose,
            'D' => Self::Debug,
            'I' => Self::Info,
            'W' => Self::Warn,
            'E' => Self::Error,
            'F' => Self::Fatal,
            _ => Self::Unknown,
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Verbose => "V",
            Self::Debug => "D",
            Self::Info => "I",
            Self::Warn => "W",
            Self::Error => "E",
            Self::Fatal => "F",
            Self::Unknown => "?",
        }
    }

    #[must_use]
    pub fn ios_label(self) -> &'static str {
        match self {
            Self::Verbose => "N",
            Self::Debug => "D",
            Self::Info => "I",
            Self::Warn => "W",
            Self::Error => "E",
            Self::Fatal => "F",
            Self::Unknown => "?",
        }
    }

    #[must_use]
    pub fn full_label(self) -> &'static str {
        match self {
            Self::Verbose => "Verbose",
            Self::Debug => "Debug",
            Self::Info => "Info",
            Self::Warn => "Warning",
            Self::Error => "Error",
            Self::Fatal => "Fatal",
            Self::Unknown => "Unknown",
        }
    }

    #[must_use]
    pub fn ios_full_label(self) -> &'static str {
        match self {
            Self::Verbose => "Notice",
            Self::Debug => "Debug",
            Self::Info => "Info",
            Self::Warn => "Warning",
            Self::Error => "Error",
            Self::Fatal => "Fault",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: u64,
    pub date: String,
    pub time: String,
    pub datetime: String,
    pub pid: u32,
    pub tid: u32,
    pub level: LogLevel,
    pub tag: String,
    pub message: String,
    pub raw: String,
}

impl Default for LogEntry {
    fn default() -> Self {
        Self {
            id: 0,
            date: String::new(),
            time: String::new(),
            datetime: String::new(),
            pid: 0,
            tid: 0,
            level: LogLevel::Unknown,
            tag: String::new(),
            message: String::new(),
            raw: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_char_returns_correct_level() {
        assert_eq!(LogLevel::from_char('V'), LogLevel::Verbose);
        assert_eq!(LogLevel::from_char('D'), LogLevel::Debug);
        assert_eq!(LogLevel::from_char('I'), LogLevel::Info);
        assert_eq!(LogLevel::from_char('W'), LogLevel::Warn);
        assert_eq!(LogLevel::from_char('E'), LogLevel::Error);
        assert_eq!(LogLevel::from_char('F'), LogLevel::Fatal);
        assert_eq!(LogLevel::from_char('X'), LogLevel::Unknown);
    }
}
