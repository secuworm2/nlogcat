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
}

impl Drop for AdbManager {
    fn drop(&mut self) {
        self.stop_stream();
    }
}
