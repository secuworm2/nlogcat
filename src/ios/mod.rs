pub mod checker;
pub mod device;
pub mod manager;
pub mod parser;
pub mod streamer;

pub use checker::ItunesChecker;
pub use device::{list_ios_devices, resolve_ios_bin_dir};
pub use manager::IosManager;
pub use streamer::IosStreamer;

#[derive(thiserror::Error, Debug)]
pub enum IosError {
    #[error("assets/bin 폴더를 찾을 수 없습니다. idevicesyslog.exe가 exe 옆 assets/bin/ 폴더에 있어야 합니다.")]
    BinDirNotFound,

    #[error("프로세스 실행 실패: {0}")]
    SpawnFailed(#[from] std::io::Error),

    #[error("연결된 iOS 기기가 없습니다.")]
    NoDevice,

    #[error("기기에서 신뢰(Trust) 버튼을 눌러주세요.")]
    TrustRequired,

    #[error("iTunes(Apple Mobile Device Support)가 설치되어 있지 않습니다.")]
    ItunesNotFound,

    #[error("로그 스트림이 예기치 않게 종료되었습니다.")]
    StreamClosed,
}
