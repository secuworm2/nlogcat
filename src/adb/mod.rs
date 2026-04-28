pub mod device;
pub mod manager;
pub mod parser;
pub mod streamer;

pub use device::{get_device_model, list_devices};
pub use parser::LogParser;

#[derive(thiserror::Error, Debug)]
pub enum AdbError {
    #[error("ADB 실행 파일을 찾을 수 없습니다: {path}")]
    NotFound { path: String },

    #[error("ADB 프로세스 실행 실패: {0}")]
    SpawnFailed(#[from] std::io::Error),

    #[error("디바이스가 연결되어 있지 않습니다")]
    NoDevice,

    #[error("스트림이 예기치 않게 종료되었습니다")]
    StreamClosed,
}
