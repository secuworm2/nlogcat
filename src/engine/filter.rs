use std::collections::HashMap;

use crate::model::filter_state::SearchField;
use crate::model::{FilterState, LogBuffer, LogEntry};

pub struct FilterEngine;

impl FilterEngine {
    #[must_use]
    pub fn matches(
        entry: &LogEntry,
        filter: &FilterState,
        pid_map: &HashMap<u32, String>,
    ) -> bool {
        if !filter.levels.contains(&entry.level) {
            return false;
        }

        if !filter.search_query.is_empty() {
            let q = filter.search_query.as_str();
            let pkg = pid_map.get(&entry.pid).map(String::as_str).unwrap_or("");

            let found = match &filter.search_field {
                SearchField::All => {
                    if filter.case_sensitive {
                        entry.tag.contains(q)
                            || entry.message.contains(q)
                            || entry.pid.to_string().contains(q)
                            || pkg.contains(q)
                    } else {
                        let q_low = q.to_lowercase();
                        entry.tag.to_lowercase().contains(&q_low)
                            || entry.message.to_lowercase().contains(&q_low)
                            || entry.pid.to_string().contains(&q_low)
                            || pkg.to_lowercase().contains(&q_low)
                    }
                }
                SearchField::Tag => {
                    if filter.case_sensitive {
                        entry.tag.contains(q)
                    } else {
                        entry.tag.to_lowercase().contains(&q.to_lowercase())
                    }
                }
                SearchField::Pid => entry.pid.to_string().contains(q),
                SearchField::Package => {
                    if filter.case_sensitive {
                        pkg.contains(q)
                    } else {
                        pkg.to_lowercase().contains(&q.to_lowercase())
                    }
                }
                SearchField::Message => {
                    if filter.case_sensitive {
                        entry.message.contains(q)
                    } else {
                        entry.message.to_lowercase().contains(&q.to_lowercase())
                    }
                }
            };

            if !found {
                return false;
            }
        }

        true
    }

    #[must_use]
    pub fn compute_indices(
        buffer: &LogBuffer,
        filter: &FilterState,
        pid_map: &HashMap<u32, String>,
    ) -> Vec<usize> {
        buffer
            .entries()
            .iter()
            .enumerate()
            .filter_map(|(i, e)| Self::matches(e, filter, pid_map).then_some(i))
            .collect()
    }
}
