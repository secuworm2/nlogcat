use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use std::os::windows::process::CommandExt;
use tokio::sync::mpsc::Sender;

use crate::ios::{IosError, IosStreamer};
use crate::model::LogEntry;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
// Trust 오류는 프로세스 시작 직후 stderr에 즉시 출력되므로 500ms 이내에 확인할 수 있다.
const TRUST_CHECK_TIMEOUT_MS: u64 = 500;

pub struct IosManager {
    bin_dir: PathBuf,
    stream_proc: Option<std::process::Child>,
    streamer: Option<IosStreamer>,
}

impl IosManager {
    #[must_use]
    pub fn new(bin_dir: PathBuf) -> Self {
        Self { bin_dir, stream_proc: None, streamer: None }
    }

    /// `idevicesyslog.exe -u <udid>` 를 스폰하고 로그 스트리밍을 시작한다.
    ///
    /// 500ms 이내에 stderr에서 Trust 오류가 감지되면 프로세스를 즉시 종료하고
    /// `IosError::TrustRequired` 를 반환한다.
    ///
    /// # Errors
    /// 프로세스 스폰 실패 또는 Trust 오류 시 `IosError`를 반환한다.
    pub fn start_stream(&mut self, udid: &str, tx: Sender<LogEntry>) -> Result<(), IosError> {
        self.stop_stream();

        let exe = self.bin_dir.join("idevicesyslog.exe");
        let mut child = std::process::Command::new(&exe)
            .args(["-u", udid])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()?;

        let stderr = child.stderr.take().ok_or_else(|| {
            IosError::SpawnFailed(std::io::Error::other("stderr pipe unavailable"))
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            IosError::SpawnFailed(std::io::Error::other("stdout pipe unavailable"))
        })?;

        // Trust 오류 감지: 별도 스레드에서 첫 번째 stderr 줄을 읽은 후 채널로 전달
        let (err_tx, err_rx) = std::sync::mpsc::channel::<String>();
        std::thread::spawn(move || {
            use std::io::BufRead;
            let reader = std::io::BufReader::new(stderr);
            if let Some(Ok(line)) = reader.lines().next() {
                let _ = err_tx.send(line);
            }
        });

        if let Ok(line) = err_rx.recv_timeout(Duration::from_millis(TRUST_CHECK_TIMEOUT_MS)) {
            if line.contains("Could not connect to lockdownd") {
                let _ = child.kill();
                return Err(IosError::TrustRequired);
            }
        }

        self.streamer = Some(IosStreamer::spawn(stdout, tx));
        self.stream_proc = Some(child);
        Ok(())
    }

    /// 스트리머 스레드를 정리하고 자식 프로세스를 종료한다.
    pub fn stop_stream(&mut self) {
        drop(self.streamer.take());
        if let Some(mut proc) = self.stream_proc.take() {
            let _ = proc.kill();
        }
    }

    /// 스트리머 스레드가 현재 실행 중인지 여부를 반환한다.
    #[must_use]
    pub fn is_streaming(&self) -> bool {
        self.streamer.as_ref().is_some_and(|s| !s.is_finished())
    }

    /// 스트리머 스레드가 스폰 후 예기치 않게 종료됐는지 반환한다 (비정상 종료 감지용).
    #[must_use]
    pub fn task_finished(&self) -> bool {
        self.streamer.as_ref().is_some_and(IosStreamer::is_finished)
    }
}

impl Drop for IosManager {
    fn drop(&mut self) {
        self.stop_stream();
    }
}
