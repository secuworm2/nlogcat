use std::collections::VecDeque;

use super::log_entry::LogEntry;

#[derive(Debug, Clone)]
pub struct LogBuffer {
    entries: VecDeque<LogEntry>,
    max_size: usize,
    next_id: u64,
}

impl LogBuffer {
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_size,
            next_id: 0,
        }
    }

    pub fn push(&mut self, mut entry: LogEntry) {
        entry.id = self.next_id;
        self.next_id += 1;
        if self.entries.len() >= self.max_size {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    #[must_use]
    pub fn entries(&self) -> &VecDeque<LogEntry> {
        &self.entries
    }

    /// O(1) lookup by sequential ID. Returns None if the entry has been evicted.
    #[must_use]
    pub fn find_by_id(&self, id: u64) -> Option<&LogEntry> {
        let first_id = self.entries.front()?.id;
        if id < first_id {
            return None;
        }
        let idx = usize::try_from(id - first_id).ok()?;
        self.entries.get(idx).filter(|e| e.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_evicts_oldest_when_full() {
        let mut buf = LogBuffer::new(2);
        buf.push(LogEntry { raw: "a".into(), ..Default::default() });
        buf.push(LogEntry { raw: "b".into(), ..Default::default() });
        buf.push(LogEntry { raw: "c".into(), ..Default::default() });
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.entries()[0].raw, "b");
        assert_eq!(buf.entries()[1].raw, "c");
    }

    #[test]
    fn clear_empties_buffer() {
        let mut buf = LogBuffer::new(10);
        buf.push(LogEntry::default());
        buf.clear();
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
    }
}
