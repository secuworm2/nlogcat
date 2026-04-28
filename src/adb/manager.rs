use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use super::streamer::AdbStreamer;
use super::AdbError;
use crate::model::log_entry::LogEntry;

pub struct AdbManager {
    task: Option<JoinHandle<()>>,
    pub adb_path: PathBuf,
}

impl AdbManager {
    #[must_use]
    pub fn new(adb_path: PathBuf) -> Self {
        Self {
            task: None,
            adb_path,
        }
    }

    pub fn start_stream(
        &mut self,
        serial: &str,
        tx: mpsc::Sender<LogEntry>,
    ) -> Result<(), AdbError> {
        self.stop_stream();
        if !self.adb_path.exists() {
            return Err(AdbError::NotFound {
                path: self.adb_path.display().to_string(),
            });
        }
        let handle = AdbStreamer::spawn_stream(&self.adb_path, serial, tx);
        self.task = Some(handle);
        Ok(())
    }

    pub fn stop_stream(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }

    pub fn restart_stream(
        &mut self,
        serial: &str,
        tx: mpsc::Sender<LogEntry>,
    ) -> Result<(), AdbError> {
        self.stop_stream();
        self.start_stream(serial, tx)
    }

    pub fn resolve_adb_path(custom: Option<&str>) -> Result<PathBuf, AdbError> {
        if let Some(path_str) = custom {
            let p = PathBuf::from(path_str);
            if p.exists() {
                return Ok(p);
            }
            return Err(AdbError::NotFound {
                path: path_str.to_string(),
            });
        }

        let adb_name = if cfg!(windows) { "adb.exe" } else { "adb" };
        if let Some(path_var) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let candidate = dir.join(adb_name);
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }

        Err(AdbError::NotFound {
            path: adb_name.to_string(),
        })
    }

    /// 스트림 task가 현재 실행 중인지 여부를 반환한다.
    #[must_use]
    pub fn is_streaming(&self) -> bool {
        self.task.as_ref().is_some_and(|t| !t.is_finished())
    }

    /// 스트림 task가 스폰된 후 종료된 상태인지 반환한다 (비정상 종료 감지용).
    #[must_use]
    pub fn task_finished(&self) -> bool {
        self.task.as_ref().is_some_and(JoinHandle::is_finished)
    }
}

impl Drop for AdbManager {
    fn drop(&mut self) {
        self.stop_stream();
    }
}
