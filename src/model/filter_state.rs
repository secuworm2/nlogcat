use std::collections::HashSet;

use super::log_entry::LogLevel;

#[derive(Debug, Clone)]
pub struct FilterState {
    pub levels: HashSet<LogLevel>,
    pub tag_includes: Vec<String>,
    pub pid_filter: Option<u32>,
    pub search_query: String,
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
            tag_includes: Vec::new(),
            pid_filter: None,
            search_query: String::new(),
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
