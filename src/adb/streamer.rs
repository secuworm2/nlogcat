use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use super::LogParser;
use crate::model::log_entry::LogEntry;

pub struct AdbStreamer;

impl AdbStreamer {
    #[must_use]
    pub fn spawn_stream(
        adb_path: &Path,
        serial: &str,
        tx: mpsc::Sender<LogEntry>,
    ) -> JoinHandle<()> {
        let adb_path = adb_path.to_path_buf();
        let serial = serial.to_string();
        tokio::spawn(async move {
            let Ok(mut child) = tokio::process::Command::new(&adb_path)
                .args(["-s", &serial, "logcat", "-v", "threadtime"])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .spawn()
            else {
                return;
            };

            let Some(stdout) = child.stdout.take() else {
                return;
            };

            let mut lines = BufReader::new(stdout).lines();

            while let Ok(Some(line)) = lines.next_line().await {
                let entry = LogParser::parse_line(&line, 0);
                // 채널 포화 시 로그 드롭 (백프레셔)
                let _ = tx.try_send(entry);
            }
        })
    }
}
