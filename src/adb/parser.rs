use crate::model::log_entry::{LogEntry, LogLevel};

pub struct LogParser;

impl LogParser {
    #[must_use]
    pub fn parse_line(line: &str, id: u64) -> LogEntry {
        Self::try_parse(line, id).unwrap_or_else(|| LogEntry {
            id,
            level: LogLevel::Unknown,
            raw: line.to_string(),
            ..Default::default()
        })
    }

    fn try_parse(line: &str, id: u64) -> Option<LogEntry> {
        let bytes = line.as_bytes();

        if bytes.len() < 20 {
            return None;
        }

        // Validate date format: MM-DD
        if !bytes[0].is_ascii_digit()
            || !bytes[1].is_ascii_digit()
            || bytes[2] != b'-'
            || !bytes[3].is_ascii_digit()
            || !bytes[4].is_ascii_digit()
        {
            return None;
        }
        let date = line[0..5].to_string();

        if bytes[5] != b' ' {
            return None;
        }

        // Validate time format: HH:MM:SS.mmm (positions 6..18)
        if bytes[8] != b':' || bytes[11] != b':' || bytes[14] != b'.' {
            return None;
        }
        let time = line[6..18].to_string();
        let datetime = format!("{date} {time}");

        let mut pos = 18_usize;

        // Skip whitespace, parse PID
        while pos < bytes.len() && bytes[pos] == b' ' {
            pos += 1;
        }
        let pid_start = pos;
        while pos < bytes.len() && bytes[pos].is_ascii_digit() {
            pos += 1;
        }
        if pos == pid_start {
            return None;
        }
        let pid: u32 = line[pid_start..pos].parse().ok()?;

        // Skip whitespace, parse TID
        while pos < bytes.len() && bytes[pos] == b' ' {
            pos += 1;
        }
        let tid_start = pos;
        while pos < bytes.len() && bytes[pos].is_ascii_digit() {
            pos += 1;
        }
        if pos == tid_start {
            return None;
        }
        let tid: u32 = line[tid_start..pos].parse().ok()?;

        // Skip whitespace, parse level char
        while pos < bytes.len() && bytes[pos] == b' ' {
            pos += 1;
        }
        if pos >= bytes.len() {
            return None;
        }
        let level = LogLevel::from_char(char::from(bytes[pos]));
        pos += 1;

        // Skip whitespace, then find tag and message separated by ": "
        while pos < bytes.len() && bytes[pos] == b' ' {
            pos += 1;
        }
        let tag_and_message = &line[pos..];

        // ADB가 버전마다 공백 개수가 다르므로 ": " 기준으로 tag/message 분리
        let sep_pos = tag_and_message.find(": ")?;
        let tag = tag_and_message[..sep_pos].trim_end().to_string();
        let message = tag_and_message[sep_pos + 2..].to_string();

        Some(LogEntry {
            id,
            date,
            time,
            datetime,
            pid,
            tid,
            level,
            tag,
            message,
            raw: line.to_string(),
        })
    }
}
