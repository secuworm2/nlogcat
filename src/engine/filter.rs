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
        query_lc: Option<&str>,
    ) -> bool {
        if !filter.levels.contains(&entry.level) {
            return false;
        }

        if !filter.search_query.is_empty() {
            let q = filter.search_query.as_str();
            let pkg = pid_map.get(&entry.pid).map_or("", String::as_str);

            let found = match &filter.search_field {
                SearchField::All => {
                    if filter.case_sensitive {
                        entry.tag.contains(q)
                            || entry.message.contains(q)
                            || entry.pid.to_string().contains(q)
                            || pkg.contains(q)
                    } else {
                        let ql = query_lc.unwrap_or(q);
                        entry.tag.to_lowercase().contains(ql)
                            || entry.message.to_lowercase().contains(ql)
                            || entry.pid.to_string().contains(ql)
                            || pkg.to_lowercase().contains(ql)
                    }
                }
                SearchField::Tag => {
                    if filter.case_sensitive {
                        entry.tag.contains(q)
                    } else {
                        entry.tag.to_lowercase().contains(query_lc.unwrap_or(q))
                    }
                }
                SearchField::Pid => entry.pid.to_string().contains(q),
                SearchField::Package => {
                    if filter.case_sensitive {
                        pkg.contains(q)
                    } else {
                        pkg.to_lowercase().contains(query_lc.unwrap_or(q))
                    }
                }
                SearchField::Message => {
                    if filter.case_sensitive {
                        entry.message.contains(q)
                    } else {
                        entry.message.to_lowercase().contains(query_lc.unwrap_or(q))
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
        let q_low = if !filter.case_sensitive && !filter.search_query.is_empty() {
            Some(filter.search_query.to_lowercase())
        } else {
            None
        };
        buffer
            .entries()
            .iter()
            .enumerate()
            .filter_map(|(i, e)| Self::matches(e, filter, pid_map, q_low.as_deref()).then_some(i))
            .collect()
    }
}
