use crate::model::{LogEntry, LogLevel};

pub struct IosLogParser;

impl IosLogParser {
    /// `idevicesyslog` stdout 한 줄을 파싱해 `LogEntry`로 반환한다.
    ///
    /// 파싱 실패 시 `LogLevel::Unknown`과 원본 문자열을 담은 엔트리를 반환한다.
    #[must_use]
    pub fn parse_line(line: &str, id: u64) -> LogEntry {
        Self::try_parse(line, id).unwrap_or_else(|| LogEntry {
            id,
            level: LogLevel::Unknown,
            raw: line.to_owned(),
            message: line.to_owned(),
            ..Default::default()
        })
    }

    fn try_parse(line: &str, id: u64) -> Option<LogEntry> {
        // iOS syslog 포맷: "Dec 15 14:23:45.123 hostname ProcessName[1234] <Error>: message"
        // "] <Level>:" 패턴으로 역방향 파싱한다.

        let bracket_close = find_bracket_angle(line)?;
        let after_sep = &line[bracket_close + 3..]; // "] <" 이후
        let gt_colon = after_sep.find(">:")?;
        let level_str = &after_sep[..gt_colon];
        let message_start = bracket_close + 3 + gt_colon + 2;
        let message = line[message_start..].trim_start().to_owned();

        // "[PID]" 추출
        let pid_open = line[..bracket_close].rfind('[')?;
        let pid: u32 = line[pid_open + 1..bracket_close].parse().ok()?;

        // 프로세스 이름: "[PID]" 앞 마지막 단어
        let before_pid = line[..pid_open].trim_end();
        let proc_start = before_pid.rfind(' ').map_or(0, |i| i + 1);
        let process = before_pid[proc_start..].to_owned();
        if process.is_empty() {
            return None;
        }

        // 타임스탬프: 앞 3개 토큰 (MMM DD HH:MM:SS...)
        let before_process = line[..proc_start].trim();
        let mut parts = before_process.splitn(4, ' ');
        let month = parts.next()?;
        let day = parts.next()?;
        let time_raw = parts.next()?;
        let date = format!("{month} {day}");
        // timezone offset 제거: "14:23:45.123456+0000" → "14:23:45.123456"
        let time = split_timezone(time_raw).to_owned();
        let datetime = format!("{date} {time}");

        Some(LogEntry {
            id,
            date,
            time,
            datetime,
            pid,
            tid: 0,
            level: map_ios_level(level_str),
            tag: process,
            message,
            raw: line.to_owned(),
        })
    }
}

/// "] <Level>:" 패턴의 위치(']' 인덱스)를 반환한다.
fn find_bracket_angle(line: &str) -> Option<usize> {
    let mut start = 0;
    while let Some(pos) = line[start..].find("] <") {
        let abs = start + pos;
        let after = &line[abs + 3..];
        if let Some(gt) = after.find(">:") {
            let level_candidate = &after[..gt];
            if !level_candidate.is_empty() && level_candidate.chars().all(char::is_alphabetic) {
                return Some(abs);
            }
        }
        start = abs + 1;
    }
    None
}

fn split_timezone(time_raw: &str) -> &str {
    if let Some(idx) = time_raw.find('+') {
        return &time_raw[..idx];
    }
    time_raw
}

fn map_ios_level(s: &str) -> LogLevel {
    match s {
        "Notice" | "Default" => LogLevel::Verbose,
        "Debug" => LogLevel::Debug,
        "Info" => LogLevel::Info,
        "Warning" => LogLevel::Warn,
        "Error" => LogLevel::Error,
        "Fault" => LogLevel::Fatal,
        _ => LogLevel::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_ios_line() {
        let line = "Dec 15 14:23:45.123 iPhone MyApp[1234] <Error>: Something went wrong";
        let entry = IosLogParser::parse_line(line, 1);
        assert_eq!(entry.level, LogLevel::Error);
        assert_eq!(entry.tag, "MyApp");
        assert_eq!(entry.pid, 1234);
        assert_eq!(entry.message, "Something went wrong");
        assert_eq!(entry.date, "Dec 15");
        assert_eq!(entry.time, "14:23:45.123");
    }

    #[test]
    fn parses_notice_as_verbose() {
        let line = "Jan  1 00:00:01.000 device kernel[0] <Notice>: boot";
        let entry = IosLogParser::parse_line(line, 2);
        assert_eq!(entry.level, LogLevel::Verbose);
    }

    #[test]
    fn fallback_on_invalid_line() {
        let line = "this is not a valid syslog line";
        let entry = IosLogParser::parse_line(line, 3);
        assert_eq!(entry.level, LogLevel::Unknown);
        assert_eq!(entry.raw, line);
    }
}
