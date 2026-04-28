use crate::model::{FilterState, LogBuffer, LogEntry};

pub struct FilterEngine;

impl FilterEngine {
    /// Returns true if the entry passes all active filter criteria.
    /// Priority: level → tag (OR) → PID → search query (AND).
    #[must_use]
    pub fn matches(entry: &LogEntry, filter: &FilterState) -> bool {
        if !filter.levels.contains(&entry.level) {
            return false;
        }

        if !filter.tag_includes.is_empty() {
            let matched = if filter.case_sensitive {
                filter.tag_includes.iter().any(|t| entry.tag.contains(t.as_str()))
            } else {
                let lower = entry.tag.to_lowercase();
                filter
                    .tag_includes
                    .iter()
                    .any(|t| lower.contains(t.to_lowercase().as_str()))
            };
            if !matched {
                return false;
            }
        }

        if let Some(pid) = filter.pid_filter {
            if entry.pid != pid {
                return false;
            }
        }

        if !filter.search_query.is_empty() {
            let found = if filter.case_sensitive {
                entry.message.contains(filter.search_query.as_str())
                    || entry.tag.contains(filter.search_query.as_str())
            } else {
                let q = filter.search_query.to_lowercase();
                entry.message.to_lowercase().contains(&q)
                    || entry.tag.to_lowercase().contains(&q)
            };
            if !found {
                return false;
            }
        }

        true
    }

    /// Scans the entire buffer and returns indices of entries that pass the filter.
    #[must_use]
    pub fn compute_indices(buffer: &LogBuffer, filter: &FilterState) -> Vec<usize> {
        buffer
            .entries()
            .iter()
            .enumerate()
            .filter_map(|(i, e)| Self::matches(e, filter).then_some(i))
            .collect()
    }

    /// Appends `entry_index` to `indices` if the entry matches the filter.
    /// Used for incremental updates when a single new entry is added to the buffer.
    pub fn append_if_matches(
        entry: &LogEntry,
        filter: &FilterState,
        indices: &mut Vec<usize>,
        entry_index: usize,
    ) {
        if Self::matches(entry, filter) {
            indices.push(entry_index);
        }
    }
}
