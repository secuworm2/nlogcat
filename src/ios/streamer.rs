use std::io::BufRead;
use std::process::ChildStdout;

use tokio::sync::mpsc::Sender;

use crate::ios::parser::IosLogParser;
use crate::model::LogEntry;

pub struct IosStreamer {
    handle: std::thread::JoinHandle<()>,
}

impl IosStreamer {
    /// `idevicesyslog` stdout을 읽어 `LogEntry`를 채널로 전송하는 스레드를 스폰한다.
    ///
    /// 프로세스가 종료(kill)되면 stdout EOF를 감지해 스레드가 자연 종료된다.
    #[must_use]
    pub fn spawn(stdout: ChildStdout, tx: Sender<LogEntry>) -> Self {
        let handle = std::thread::spawn(move || {
            let reader = std::io::BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        let entry = IosLogParser::parse_line(&line, 0);
                        if tx.blocking_send(entry).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        Self { handle }
    }

    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }
}
