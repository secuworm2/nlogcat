use std::collections::HashSet;

use super::log_entry::LogLevel;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SearchField {
    #[default]
    All,
    Tag,
    Pid,
    Package,
    Message,
}

impl SearchField {
    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "전체",
            Self::Tag => "태그",
            Self::Pid => "PID",
            Self::Package => "패키지",
            Self::Message => "메시지",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilterState {
    pub levels: HashSet<LogLevel>,
    pub search_query: String,
    pub search_field: SearchField,
    pub case_sensitive: bool,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            levels: HashSet::from([
                LogLevel::Verbose,
                LogLevel::Debug,
                LogLevel::Info,
                LogLevel::Warn,
                LogLevel::Error,
                LogLevel::Fatal,
            ]),
            search_query: String::new(),
            search_field: SearchField::All,
            case_sensitive: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_contains_all_six_levels() {
        let state = FilterState::default();
        assert_eq!(state.levels.len(), 6);
        assert!(state.levels.contains(&LogLevel::Verbose));
        assert!(state.levels.contains(&LogLevel::Debug));
        assert!(state.levels.contains(&LogLevel::Info));
        assert!(state.levels.contains(&LogLevel::Warn));
        assert!(state.levels.contains(&LogLevel::Error));
        assert!(state.levels.contains(&LogLevel::Fatal));
    }
}
